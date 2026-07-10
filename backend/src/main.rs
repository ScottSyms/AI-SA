mod agent;
mod config;
mod db;
mod llm;
mod routes;
mod skill;
mod speech;
mod state;
mod tts;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::db::Database;
use crate::llm::LlmClient;
use crate::skill::load_skills;
use crate::state::AppState;
use crate::tts::OpenAiTtsClient;

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

    let tts = config.openai_api_key.as_ref().map(|key| {
        tracing::info!(
            model = %config.openai_tts_model,
            voice = %config.openai_tts_voice,
            "OpenAI TTS enabled"
        );
        OpenAiTtsClient::new(
            key.clone(),
            config.openai_tts_model.clone(),
            config.openai_tts_voice.clone(),
        )
    });

    // Build router
    let state = AppState::new(config.clone(), db, skills, llm, tts);

    let app = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/skills", get(routes::list_skills))
        .route("/api/skills/{name}/layer-data", get(routes::skill_layer_data))
        .route("/api/query", post(routes::execute_query))
        .route("/api/agent", post(routes::agent_handler))
        .route("/api/tts", post(routes::tts_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Bind with SO_REUSEADDR to avoid "Address already in use" on restart
    let addr: std::net::SocketAddr = config
        .bind_addr
        .parse()
        .expect("invalid bind address");
    let socket = socket2::Socket::new(
        socket2::Domain::for_address(addr),
        socket2::Type::STREAM,
        Some(socket2::Protocol::TCP),
    )
    .expect("failed to create socket");
    socket.set_reuse_address(true).expect("failed to set SO_REUSEADDR");
    socket.bind(&addr.into()).unwrap_or_else(|e| {
        panic!("failed to bind to {addr}: {e}");
    });
    socket.listen(1024).expect("failed to listen");
    let std_listener: std::net::TcpListener = socket.into();
    std_listener
        .set_nonblocking(true)
        .expect("failed to set nonblocking");
    let listener = tokio::net::TcpListener::from_std(std_listener)
        .expect("failed to create tokio listener");
    tracing::info!(addr = %config.bind_addr, "backend listening");
    axum::serve(listener, app).await.unwrap();
}
