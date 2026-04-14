use duckdb::types::ValueRef;
use duckdb::Connection;
use serde_json::{json, Value as JsonValue};
use std::path::Path;
use std::sync::Mutex;

use crate::skill::SkillManifest;

/// Maximum number of rows any ad hoc query can return.
const MAX_QUERY_LIMIT: usize = 500;

/// Server-side DuckDB wrapper.
pub struct Database {
    conn: Mutex<Connection>,
}

/// Validate a SQL query for safety. Returns Ok(()) if safe, Err(reason) if not.
pub fn validate_sql(sql: &str) -> Result<(), String> {
    let trimmed = sql.trim();
    let upper = trimmed.to_uppercase();

    // Reject empty queries
    if trimmed.is_empty() {
        return Err("empty query".into());
    }

    // Must start with SELECT or WITH (CTEs)
    if !upper.starts_with("SELECT") && !upper.starts_with("WITH") {
        return Err("only SELECT and WITH (CTE) queries are allowed".into());
    }

    // Reject write operations anywhere in the query (not just at start)
    let forbidden = [
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "DROP ",
        "ALTER ",
        "CREATE ",
        "TRUNCATE ",
        "COPY ",
        "ATTACH ",
        "DETACH ",
        "PRAGMA ",
        "LOAD ",
        "INSTALL ",
        "EXPORT ",
    ];
    for keyword in &forbidden {
        if upper.contains(keyword) {
            return Err(format!("forbidden keyword: {}", keyword.trim()));
        }
    }

    // Reject multiple statements (semicolons followed by non-whitespace)
    let without_strings = strip_string_literals(trimmed);
    let semicolons: Vec<_> = without_strings.match_indices(';').collect();
    for (pos, _) in &semicolons {
        let after = without_strings[pos + 1..].trim();
        if !after.is_empty() {
            return Err("multiple statements not allowed".into());
        }
    }

    // Must contain LIMIT
    if !upper.contains("LIMIT") {
        return Err("query must contain a LIMIT clause".into());
    }

    // Enforce maximum LIMIT value
    if let Some(limit_val) = extract_limit_value(&upper) {
        if limit_val > MAX_QUERY_LIMIT {
            return Err(format!(
                "LIMIT {} exceeds maximum allowed ({})",
                limit_val, MAX_QUERY_LIMIT
            ));
        }
    }

    Ok(())
}

/// Execute a validated read-only query. Validates before executing.
pub fn safe_query(db: &Database, sql: &str) -> Result<JsonValue, String> {
    validate_sql(sql)?;
    db.query(sql)
}

/// Strip string literals from SQL to avoid false positives in keyword detection.
fn strip_string_literals(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\'' {
            // Skip until matching quote
            i += 1;
            while i < chars.len() {
                if chars[i] == '\'' {
                    if i + 1 < chars.len() && chars[i + 1] == '\'' {
                        i += 2; // escaped quote
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            result.push_str("''"); // placeholder
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Extract the numeric LIMIT value from a SQL string (uppercase).
fn extract_limit_value(upper_sql: &str) -> Option<usize> {
    // Find last occurrence of LIMIT (to handle subqueries)
    let idx = upper_sql.rfind("LIMIT")?;
    let after = upper_sql[idx + 5..].trim_start();
    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

impl Database {
    /// Create an in-memory DuckDB connection.
    pub fn new() -> Result<Self, duckdb::Error> {
        let conn = Connection::open_in_memory()?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    /// Load parquet files matching a glob pattern into a table.
    pub fn load_parquet(&self, table_name: &str, glob_pattern: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "CREATE OR REPLACE TABLE {table_name} AS SELECT * FROM read_parquet('{glob_pattern}')"
        );
        conn.execute_batch(&sql)
            .map_err(|e| format!("failed to load parquet into {table_name}: {e}"))?;

        // Log row count
        let mut stmt = conn
            .prepare(&format!("SELECT COUNT(*) FROM {table_name}"))
            .map_err(|e| format!("count query failed: {e}"))?;
        let count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| format!("count read failed: {e}"))?;
        tracing::info!(table = table_name, rows = count, "loaded parquet data");

        Ok(())
    }

    /// Execute SQL view creation statements (from views.sql).
    pub fn register_views(&self, sql: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)
            .map_err(|e| format!("failed to register views: {e}"))
    }

    /// Execute a read-only query and return results as JSON rows.
    pub fn query(&self, sql: &str) -> Result<JsonValue, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("prepare error: {e}"))?;

        // Execute query first — column_count/column_name panic before execution
        let mut result_rows = stmt.query([]).map_err(|e| format!("query error: {e}"))?;

        // Get column info from the underlying statement via Rows::as_ref
        let (column_count, column_names) = match result_rows.as_ref() {
            Some(s) => {
                let count = s.column_count();
                let names: Vec<String> = (0..count)
                    .map(|i| s.column_name(i).map_or("?".to_string(), |v| v.to_string()))
                    .collect();
                (count, names)
            }
            None => (0, Vec::new()),
        };

        let mut rows = Vec::new();
        while let Some(row) = result_rows.next().map_err(|e| format!("row error: {e}"))? {
            let mut obj = serde_json::Map::new();
            for i in 0..column_count {
                let json_val = match row.get_ref(i) {
                    Ok(val) => value_ref_to_json(&val),
                    Err(_) => JsonValue::Null,
                };
                obj.insert(column_names[i].clone(), json_val);
            }
            rows.push(JsonValue::Object(obj));
        }

        Ok(json!({
            "columns": column_names,
            "rows": rows,
            "row_count": rows.len()
        }))
    }

    /// Execute a parameterized query with named $-prefixed parameters replaced.
    pub fn query_with_params(
        &self,
        sql_template: &str,
        params: &serde_json::Map<String, JsonValue>,
    ) -> Result<JsonValue, String> {
        // Simple parameter substitution: replace $param with the value
        let mut sql = sql_template.to_string();
        for (key, value) in params {
            let placeholder = format!("${key}");
            let replacement = match value {
                JsonValue::Null => "NULL".to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::String(s) => format!("'{}'", s.replace('\'', "''")),
                JsonValue::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            sql = sql.replace(&placeholder, &replacement);
        }

        // Replace any remaining $param placeholders with NULL
        // (parameters not provided by the caller)
        sql = replace_remaining_params(&sql);

        self.query(&sql)
    }

    /// Initialize a skill: load its data source and register its views.
    pub fn init_skill(&self, skill: &SkillManifest, project_root: &Path) -> Result<(), String> {
        // Load parquet data if specified
        if let (Some(source_type), Some(source_path)) = (&skill.source_type, &skill.source_path) {
            if source_type == "parquet" {
                let resolved = project_root.join(source_path);
                let resolved_str = resolved.to_string_lossy();
                tracing::info!(skill = %skill.name, path = %resolved_str, "loading parquet source");
                self.load_parquet(&skill.name, &resolved_str)?;
            }
        }

        // Register SQL views
        if !skill.sql_views.is_empty() {
            tracing::info!(skill = %skill.name, "registering SQL views");
            self.register_views(&skill.sql_views)?;
        }

        Ok(())
    }
}

/// Convert a DuckDB ValueRef to a serde_json Value.
fn value_ref_to_json(val: &ValueRef) -> JsonValue {
    match val {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Boolean(b) => json!(b),
        ValueRef::TinyInt(i) => json!(i),
        ValueRef::SmallInt(i) => json!(i),
        ValueRef::Int(i) => json!(i),
        ValueRef::BigInt(i) => json!(i),
        ValueRef::HugeInt(i) => json!(i.to_string()),
        ValueRef::UTinyInt(i) => json!(i),
        ValueRef::USmallInt(i) => json!(i),
        ValueRef::UInt(i) => json!(i),
        ValueRef::UBigInt(i) => json!(i),
        ValueRef::Float(f) => json!(f),
        ValueRef::Double(f) => json!(f),
        ValueRef::Text(s) => {
            // s is &[u8], convert to string
            json!(String::from_utf8_lossy(s))
        }
        ValueRef::Blob(b) => json!(format!("<blob {} bytes>", b.len())),
        ValueRef::Timestamp(_, t) => json!(t),
        ValueRef::Date32(d) => json!(d),
        ValueRef::Time64(_, t) => json!(t),
        _ => json!("unsupported"),
    }
}

/// Replace any remaining `$param_name` placeholders with NULL.
fn replace_remaining_params(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$'
            && i + 1 < chars.len()
            && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_')
        {
            // Found a $param — skip over the identifier
            i += 1; // skip $
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            result.push_str("NULL");
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}
