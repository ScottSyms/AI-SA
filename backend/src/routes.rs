use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::agent::{AgentRequest, RenderEnvelope};
use crate::db::validate_sql;
use crate::state::AppState;

// ── Health ──────────────────────────────────────────────────────────────────

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "tommy3-backend" }))
}

// ── GET /api/skills ─────────────────────────────────────────────────────────

pub async fn list_skills(State(state): State<Arc<AppState>>) -> Json<Value> {
    let skills: Vec<Value> = state
        .skills
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "source_type": s.source_type,
                "interactions": s.interactions,
                "map_layers": s.map_layers,
                "tools": s.tools,
                "domain_prompt": s.domain_prompt,
            })
        })
        .collect();

    Json(json!({
        "status": "ok",
        "skills": skills,
        "count": skills.len()
    }))
}

// ── POST /api/query ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    /// Raw SQL to execute (must be read-only)
    pub sql: Option<String>,
    /// Or: skill name + tool name + params (resolved to SQL server-side)
    pub skill: Option<String>,
    pub tool: Option<String>,
    pub params: Option<serde_json::Map<String, Value>>,
}

pub async fn execute_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Path 1: skill + tool + params → resolve SQL from skill's sql_files
    if let (Some(skill_name), Some(tool_name)) = (&req.skill, &req.tool) {
        let skill = state
            .skills
            .iter()
            .find(|s| s.name == *skill_name)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "status": "error", "error": format!("skill '{}' not found", skill_name) })),
                )
            })?;

        // Find the tool definition
        let tool = skill
            .tools
            .iter()
            .find(|t| t.name == *tool_name)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "status": "error", "error": format!("tool '{}' not found in skill '{}'", tool_name, skill_name) })),
                )
            })?;

        // Resolve SQL file
        let sql_filename = tool
            .sql_file
            .as_ref()
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": "error", "error": "tool has no SQL file" })),
                )
            })?;

        let sql_key = sql_filename.trim_end_matches(".sql");
        let sql_template = skill
            .sql_files
            .get(sql_key)
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "status": "error", "error": format!("SQL file '{}' not found", sql_filename) })),
                )
            })?;

        let params = req.params.as_ref().cloned().unwrap_or_default();
        let result = state
            .db
            .query_with_params(sql_template, &params)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "status": "error", "error": e })),
                )
            })?;

        return Ok(Json(json!({
            "status": "ok",
            "data": result
        })));
    }

    // Path 2: raw SQL (validated for safety)
    if let Some(sql) = &req.sql {
        if let Err(reason) = validate_sql(sql) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "status": "error", "error": reason })),
            ));
        }

        let result = state.db.query(sql).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "error": e })),
            )
        })?;

        return Ok(Json(json!({
            "status": "ok",
            "data": result
        })));
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({ "status": "error", "error": "provide either 'sql' or 'skill'+'tool'" })),
    ))
}

// ── POST /api/agent ─────────────────────────────────────────────────────────

pub async fn agent_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AgentRequest>,
) -> Json<Value> {
    let agent = match &state.agent {
        Some(a) => a,
        None => {
            return Json(json!({
                "status": "error",
                "error": "Agent not available — OPENAI_API_KEY not configured"
            }));
        }
    };

    tracing::info!(message = %req.message, "agent request");

    let envelope = agent.run(&req, &state.skills, &state.db).await;

    Json(serde_json::to_value(&envelope).unwrap_or_else(|e| {
        json!({
            "status": "error",
            "error": format!("serialization error: {e}")
        })
    }))
}
