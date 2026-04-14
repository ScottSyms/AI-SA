mod agent;
mod config;
mod db;
mod llm;
mod routes;
mod skill;
mod state;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::db::Database;
use crate::llm::LlmClient;
use crate::skill::load_skills;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load config
    let config = Config::load();
    tracing::info!(root = %config.project_root.display(), "project root resolved");

    // Initialize DuckDB
    let db = Database::new().expect("failed to create DuckDB connection");

    // Load skills
    let skills = load_skills(&config.skills_dir);
    tracing::info!(count = skills.len(), "skills loaded");

    // Initialize each skill in DuckDB (load data, register views)
    for skill in &skills {
        match db.init_skill(skill, &config.project_root) {
            Ok(()) => tracing::info!(skill = %skill.name, "skill initialized in DuckDB"),
            Err(e) => tracing::warn!(skill = %skill.name, error = %e, "skill init failed"),
        }
    }

    // Initialize LLM client (optional — agent works without it, returns error)
    let llm = config.openai_api_key.as_ref().map(|key| {
        tracing::info!("OpenAI API key found, agent enabled");
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
        Arc::new(LlmClient::new(key.clone()).with_model(&model))
    });

    if llm.is_none() {
        tracing::warn!("No OPENAI_API_KEY set — agent will return errors. Set it in .env");
    }

    // Build router
    let state = AppState::new(config.clone(), db, skills, llm);

    let app = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/skills", get(routes::list_skills))
        .route("/api/query", post(routes::execute_query))
        .route("/api/agent", post(routes::agent_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Serve
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind");
    tracing::info!(addr = %config.bind_addr, "backend listening");
    axum::serve(listener, app).await.unwrap();
}
