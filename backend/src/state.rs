use std::sync::Arc;

use crate::agent::Agent;
use crate::config::Config;
use crate::db::Database;
use crate::llm::LlmClient;
use crate::skill::SkillManifest;

/// Shared application state passed to all route handlers.
pub struct AppState {
    pub config: Config,
    pub db: Database,
    pub skills: Vec<SkillManifest>,
    pub agent: Option<Agent>,
}

impl AppState {
    pub fn new(
        config: Config,
        db: Database,
        skills: Vec<SkillManifest>,
        llm: Option<Arc<LlmClient>>,
    ) -> Arc<Self> {
        let agent = llm.map(|l| Agent::new(l));
        Arc::new(AppState {
            config,
            db,
            skills,
            agent,
        })
    }
}
