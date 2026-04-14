//! Agent runtime: interprets user intent, routes to skills, executes tools.
//!
//! Execution flow (from AGENTS.md §9):
//!   1. interpret intent
//!   2. resolve context (selection, geometry)
//!   3. choose skill
//!   4. try deterministic method
//!   5. fallback to ad hoc SQL
//!   6. validate
//!   7. execute
//!   8. normalize output
//!   9. return renderable result

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::{safe_query, Database};
use crate::llm::{FunctionDef, LlmClient, Message, ToolCall, ToolDef};
use crate::skill::{SkillManifest, SkillTool};

/// Maximum tool-calling iterations to prevent runaway loops.
const MAX_TOOL_ROUNDS: usize = 8;

// ── Public types ───────────────────────────────────────────────────────────

/// Incoming agent request from the frontend.
#[derive(Debug, Deserialize)]
pub struct AgentRequest {
    pub message: String,
    #[serde(default)]
    pub context: Option<AgentContext>,
}

/// Optional context sent with the agent request (selection state, viewport, etc.)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentContext {
    /// Currently selected items
    #[serde(default)]
    pub selection: Vec<SelectionItem>,
    /// Map viewport bounds [min_lon, min_lat, max_lon, max_lat]
    #[serde(default)]
    pub viewport: Option<[f64; 4]>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelectionItem {
    pub mmsi: i64,
    pub name: String,
}

/// The render envelope returned to the frontend (AGENTS.md §4).
#[derive(Debug, Clone, Serialize)]
pub struct RenderEnvelope {
    pub status: String,
    pub title: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub render_as: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl RenderEnvelope {
    pub fn text(title: &str, summary: &str) -> Self {
        RenderEnvelope {
            status: "ok".into(),
            title: title.into(),
            summary: summary.into(),
            data: None,
            render_as: vec!["panel".into()],
            columns: vec![],
            geometry: None,
            actions: vec![],
            warnings: vec![],
        }
    }

    pub fn error(msg: &str) -> Self {
        RenderEnvelope {
            status: "error".into(),
            title: "Error".into(),
            summary: msg.into(),
            data: None,
            render_as: vec!["panel".into()],
            columns: vec![],
            geometry: None,
            actions: vec![],
            warnings: vec![],
        }
    }

    pub fn with_table(mut self, columns: Vec<String>, data: Value) -> Self {
        self.columns = columns;
        self.data = Some(data);
        if !self.render_as.contains(&"table".to_string()) {
            self.render_as.push("table".into());
        }
        self
    }

    pub fn with_geometry(mut self, geojson: Value) -> Self {
        self.geometry = Some(geojson);
        if !self.render_as.contains(&"map".to_string()) {
            self.render_as.push("map".into());
        }
        self
    }
}

// ── Agent ──────────────────────────────────────────────────────────────────

pub struct Agent {
    llm: Arc<LlmClient>,
}

impl Agent {
    pub fn new(llm: Arc<LlmClient>) -> Self {
        Agent { llm }
    }

    /// Run the agent: interpret the user message, call tools as needed, return a RenderEnvelope.
    pub async fn run(
        &self,
        req: &AgentRequest,
        skills: &[SkillManifest],
        db: &Database,
    ) -> RenderEnvelope {
        // Build dynamic schema description from DuckDB metadata
        let db_schema = db.describe_schema();

        // Build system prompt from loaded skills + live schema
        let system_prompt = build_system_prompt(skills, req.context.as_ref(), &db_schema);

        // Convert skill tools to OpenAI function-calling format
        let tools = build_tool_defs(skills);

        // Add a built-in SQL query tool for ad hoc queries with dynamic schema
        let run_sql_desc = format!(
            "Execute a read-only SQL query against the DuckDB database. \
             The query MUST include a LIMIT clause. \
             \n\nDATABASE SCHEMA:\n{}\
             \nIMPORTANT: There are NO spatial extensions (no ST_Distance, no PostGIS). \
             For distance calculations, use the Euclidean approximation: \
             ORDER BY pow(lat - $target_lat, 2) + pow(lon - $target_lon, 2) ASC. \
             For accurate distance in km, use: \
             6371 * acos(cos(radians($lat1)) * cos(radians(lat)) * cos(radians(lon) - radians($lon1)) + sin(radians($lat1)) * sin(radians(lat))) as distance_km",
            db_schema
        );

        let mut all_tools = tools;
        all_tools.push(ToolDef {
            tool_type: "function".into(),
            function: FunctionDef {
                name: "run_sql".into(),
                description: run_sql_desc,
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "sql": {
                            "type": "string",
                            "description": "The SQL query to execute (must be read-only with LIMIT)"
                        }
                    },
                    "required": ["sql"]
                }),
            },
        });

        // Start conversation
        let mut messages = vec![
            Message::system(&system_prompt),
            Message::user(&req.message),
        ];

        // Tool-calling loop
        for round in 0..MAX_TOOL_ROUNDS {
            let is_last_round = round == MAX_TOOL_ROUNDS - 1;

            // On the final round, omit tools to force a text-only response
            // from whatever data has been gathered so far.
            let chat_req = if is_last_round {
                tracing::warn!("agent reached final round, forcing text-only response");
                messages.push(Message::system(
                    "You have used all available tool-calling rounds. \
                     Summarize what you have learned so far and give the best answer you can \
                     based on the data already retrieved. Do NOT request any more tool calls."
                ));
                self.llm
                    .request()
                    .messages(messages.clone())
                    .temperature(0.1)
                    .build()
            } else {
                self.llm
                    .request()
                    .messages(messages.clone())
                    .tools(all_tools.clone())
                    .temperature(0.1)
                    .build()
            };

            let response = match self.llm.chat(&chat_req).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "LLM call failed");
                    return RenderEnvelope::error(&format!("LLM error: {e}"));
                }
            };

            let choice = match response.choices.first() {
                Some(c) => c,
                None => return RenderEnvelope::error("No response from LLM"),
            };

            // If the model returned tool calls, execute them
            if let Some(ref tool_calls) = choice.message.tool_calls {
                tracing::info!(round, calls = tool_calls.len(), "agent tool calls");

                // Add assistant message with tool calls to history
                messages.push(Message::assistant_tool_calls(tool_calls.clone()));

                // Execute each tool call
                for tc in tool_calls {
                    let result = execute_tool_call(tc, skills, db).await;
                    tracing::info!(
                        tool = %tc.function.name,
                        result_len = result.len(),
                        "tool executed"
                    );
                    messages.push(Message::tool_result(&tc.id, &result));
                }

                // Continue the loop — model will process tool results
                continue;
            }

            // No tool calls — model gave a final text response
            let text = choice
                .message
                .content
                .clone()
                .unwrap_or_else(|| "No response generated.".into());

            if let Some(usage) = &response.usage {
                tracing::info!(
                    prompt_tokens = usage.prompt_tokens,
                    completion_tokens = usage.completion_tokens,
                    total_tokens = usage.total_tokens,
                    "agent complete"
                );
            }

            // Try to parse structured data from the last tool result for rendering
            let last_tool_data = extract_last_tool_data(&messages);

            return match last_tool_data {
                Some((columns, data, _row_count)) => {
                    let mut envelope = RenderEnvelope::text("Query Result", &text);
                    envelope = envelope.with_table(columns, data);

                    // Check if data has lat/lon for map rendering
                    if has_geometry_columns(&envelope.columns) {
                        if let Some(geojson) = rows_to_geojson(envelope.data.as_ref()) {
                            envelope = envelope.with_geometry(geojson);
                        }
                    }

                    envelope
                }
                None => RenderEnvelope::text("Response", &text),
            };
        }

        // This should be unreachable now (final round forces text), but keep as safety net
        RenderEnvelope::error("Agent exceeded maximum tool-calling rounds")
    }
}

// ── System prompt builder ──────────────────────────────────────────────────

fn build_system_prompt(skills: &[SkillManifest], context: Option<&AgentContext>, db_schema: &str) -> String {
    let mut prompt = format!(
        "You are a geospatial intelligence agent. You help users explore and analyze spatial data.\n\n\
         RULES:\n\
         - Use the provided tools to answer questions. Do NOT guess data.\n\
         - When querying data, always use LIMIT to bound results.\n\
         - Present results clearly with relevant details.\n\
         - If the user asks about selected vessels, use the context provided.\n\
         - Use nautical terminology where appropriate for maritime data.\n\
         - If you need to run a custom query, use the run_sql tool.\n\
         - NEVER call the same tool more than twice with the same intent. If a query fails, try a different approach or report what you know.\n\
         - Prefer FEWER tool calls. One well-crafted query is better than many exploratory ones.\n\n\
         DATABASE SCHEMA:\n\
         {db_schema}\n\
         DATABASE CONSTRAINTS:\n\
         - DuckDB with NO spatial extensions (no ST_Distance, no ST_Point, no PostGIS functions).\n\
         - For distance/proximity queries, use Euclidean approximation or haversine:\n\
           Euclidean (fast, for sorting): ORDER BY pow(lat - TARGET_LAT, 2) + pow(lon - TARGET_LON, 2) ASC\n\
           Haversine (accurate km): 6371 * acos(cos(radians(LAT1)) * cos(radians(lat)) * cos(radians(lon) - radians(LON1)) + sin(radians(LAT1)) * sin(radians(lat))) as distance_km\n\
         - For bounding box filters: WHERE lat BETWEEN min_lat AND max_lat AND lon BETWEEN min_lon AND max_lon\n\n\
         COMMON LOCATIONS (approximate coordinates):\n\
         - London: 51.5, -0.1 | New York: 40.7, -74.0 | Tokyo: 35.7, 139.7\n\
         - Singapore: 1.3, 103.8 | Dubai: 25.2, 55.3 | Rotterdam: 51.9, 4.5\n\
         - Gibraltar: 36.1, -5.3 | Panama Canal: 9.0, -79.5 | Suez Canal: 30.5, 32.3\n\
         - English Channel: 50.5, -1.0 | Mediterranean: 35.0, 18.0 | North Sea: 56.0, 3.0\n\n\
         SQL EXAMPLES:\n\
         -- Count vessels: SELECT COUNT(*) as count FROM v_latest_positions LIMIT 1\n\
         -- Vessel types: SELECT vessel_type, COUNT(*) as count FROM v_latest_positions GROUP BY vessel_type ORDER BY count DESC LIMIT 20\n\
         -- Fastest vessels: SELECT name, mmsi, speed, lat, lon, vessel_type FROM v_latest_positions ORDER BY speed DESC LIMIT 5\n\
         -- Nearest to a point: SELECT name, mmsi, lat, lon, speed, vessel_type, pow(lat - 51.5, 2) + pow(lon - (-0.1), 2) as dist_sq FROM v_latest_positions ORDER BY dist_sq ASC LIMIT 5\n\
         -- Search ports: SELECT port_name, country, lat, lon, size FROM v_all_ports WHERE port_name ILIKE '%rotterdam%' LIMIT 10\n\
         -- Ports near vessels: SELECT p.port_name, p.country, p.lat, p.lon, p.size FROM v_all_ports p ORDER BY pow(p.lat - 51.5, 2) + pow(p.lon - (-0.1), 2) ASC LIMIT 5\n\n"
    );

    // Inject each skill's domain prompt
    for skill in skills {
        if !skill.domain_prompt.is_empty() {
            prompt.push_str(&format!(
                "## Skill: {}\n{}\n\n",
                skill.name, skill.domain_prompt
            ));
        }

        // List available tools
        prompt.push_str(&format!("Available tools for skill '{}':\n", skill.name));
        for tool in &skill.tools {
            prompt.push_str(&format!("  - {}(", tool.name));
            let params: Vec<String> = tool
                .parameters
                .iter()
                .map(|p| {
                    if p.required {
                        format!("{}: {}", p.name, p.param_type)
                    } else {
                        format!("{}?: {}", p.name, p.param_type)
                    }
                })
                .collect();
            prompt.push_str(&params.join(", "));
            prompt.push_str(&format!(") — {}\n", tool.description));
        }
        prompt.push('\n');
    }

    // Inject context
    if let Some(ctx) = context {
        if !ctx.selection.is_empty() {
            prompt.push_str("CURRENT SELECTION:\n");
            for item in &ctx.selection {
                prompt.push_str(&format!("  - {} (MMSI: {})\n", item.name, item.mmsi));
            }
            prompt.push('\n');
        }
        if let Some(vp) = &ctx.viewport {
            prompt.push_str(&format!(
                "CURRENT VIEWPORT: [{:.2}, {:.2}, {:.2}, {:.2}] (min_lon, min_lat, max_lon, max_lat)\n\n",
                vp[0], vp[1], vp[2], vp[3]
            ));
        }
    }

    prompt
}

// ── Tool definition builder ────────────────────────────────────────────────

fn build_tool_defs(skills: &[SkillManifest]) -> Vec<ToolDef> {
    let mut defs = Vec::new();

    for skill in skills {
        for tool in &skill.tools {
            let params = build_json_schema_params(tool);
            defs.push(ToolDef {
                tool_type: "function".into(),
                function: FunctionDef {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: params,
                },
            });
        }
    }

    defs
}

fn build_json_schema_params(tool: &SkillTool) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in &tool.parameters {
        let json_type = match param.param_type.as_str() {
            "integer" => "integer",
            "float" | "number" | "double" => "number",
            "boolean" | "bool" => "boolean",
            _ => "string",
        };

        let mut prop = json!({ "type": json_type });
        if let Some(default) = &param.default {
            prop["description"] = json!(format!("Default: {}", default));
        }
        properties.insert(param.name.clone(), prop);

        if param.required {
            required.push(json!(param.name));
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

// ── Tool execution ─────────────────────────────────────────────────────────

async fn execute_tool_call(
    tc: &ToolCall,
    skills: &[SkillManifest],
    db: &Database,
) -> String {
    let args: Value = match serde_json::from_str(&tc.function.arguments) {
        Ok(v) => v,
        Err(e) => return json!({"error": format!("bad arguments: {e}")}).to_string(),
    };

    // Handle built-in run_sql tool
    if tc.function.name == "run_sql" {
        let sql = args.get("sql").and_then(|v| v.as_str()).unwrap_or("");
        return match safe_query(db, sql) {
            Ok(result) => result.to_string(),
            Err(e) => json!({"error": e}).to_string(),
        };
    }

    // Find the tool in loaded skills
    for skill in skills {
        if let Some(tool) = skill.tools.iter().find(|t| t.name == tc.function.name) {
            // Get SQL template
            let sql_filename = match &tool.sql_file {
                Some(f) => f,
                None => return json!({"error": "tool has no SQL file"}).to_string(),
            };
            let sql_key = sql_filename.trim_end_matches(".sql");
            let sql_template = match skill.sql_files.get(sql_key) {
                Some(s) => s,
                None => {
                    return json!({"error": format!("SQL file '{}' not found", sql_filename)})
                        .to_string()
                }
            };

            // Build params map from tool call arguments
            let params = match args.as_object() {
                Some(obj) => {
                    let mut map = serde_json::Map::new();
                    for (k, v) in obj {
                        map.insert(k.clone(), v.clone());
                    }
                    map
                }
                None => serde_json::Map::new(),
            };

            return match db.query_with_params(sql_template, &params) {
                Ok(result) => result.to_string(),
                Err(e) => json!({"error": e}).to_string(),
            };
        }
    }

    json!({"error": format!("unknown tool: {}", tc.function.name)}).to_string()
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Extract the last tool result data from conversation for rendering.
fn extract_last_tool_data(messages: &[Message]) -> Option<(Vec<String>, Value, usize)> {
    // Walk backwards to find the last tool result
    for msg in messages.iter().rev() {
        if msg.role == "tool" {
            if let Some(content) = &msg.content {
                if let Ok(parsed) = serde_json::from_str::<Value>(content) {
                    if let Some(columns) = parsed.get("columns") {
                        let cols: Vec<String> = columns
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let rows = parsed.get("rows").cloned().unwrap_or(json!([]));
                        let row_count = parsed
                            .get("row_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as usize;
                        return Some((cols, rows, row_count));
                    }
                }
            }
        }
    }
    None
}

fn has_geometry_columns(columns: &[String]) -> bool {
    let has_lat = columns.iter().any(|c| c == "lat" || c == "latitude");
    let has_lon = columns.iter().any(|c| c == "lon" || c == "longitude" || c == "lng");
    has_lat && has_lon
}

/// Convert rows with lat/lon to GeoJSON FeatureCollection.
fn rows_to_geojson(data: Option<&Value>) -> Option<Value> {
    let rows = data?.as_array()?;
    let features: Vec<Value> = rows
        .iter()
        .filter_map(|row| {
            let lat = row.get("lat").and_then(|v| v.as_f64())?;
            let lon = row.get("lon").and_then(|v| v.as_f64())?;
            Some(json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [lon, lat]
                },
                "properties": row
            }))
        })
        .collect();

    if features.is_empty() {
        return None;
    }

    Some(json!({
        "type": "FeatureCollection",
        "features": features
    }))
}
