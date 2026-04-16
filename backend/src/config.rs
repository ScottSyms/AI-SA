use std::path::PathBuf;

/// Application configuration resolved from environment and conventions.
#[derive(Debug, Clone)]
pub struct Config {
    /// Root of the project (parent of backend/)
    pub project_root: PathBuf,
    /// Path to skills/ directory
    pub skills_dir: PathBuf,
    /// Path to data/ directory
    #[allow(dead_code)]
    pub data_dir: PathBuf,
    /// Server bind address
    pub bind_addr: String,
    /// OpenAI API key (optional for Phase 3)
    pub openai_api_key: Option<String>,
    /// OpenAI TTS model name
    pub openai_tts_model: String,
    /// OpenAI TTS voice name
    pub openai_tts_voice: String,
}

impl Config {
    /// Load configuration from environment variables and conventions.
    /// Expects to be run from the backend/ directory or with PROJECT_ROOT set.
    pub fn load() -> Self {
        let _ = dotenvy::dotenv(); // .env in project root or backend/

        let project_root = std::env::var("PROJECT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Assume binary runs from backend/ or project root
                let cwd = std::env::current_dir().expect("cannot determine cwd");
                if cwd.join("skills").is_dir() {
                    cwd
                } else if cwd.join("../skills").is_dir() {
                    cwd.join("..")
                } else {
                    cwd
                }
            })
            .canonicalize()
            .expect("cannot canonicalize project root");

        let skills_dir = project_root.join("skills");
        let data_dir = project_root.join("data");

        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        let openai_tts_model =
            std::env::var("OPENAI_TTS_MODEL").unwrap_or_else(|_| "gpt-4o-mini-tts".to_string());
        let openai_tts_voice =
            std::env::var("OPENAI_TTS_VOICE").unwrap_or_else(|_| "alloy".to_string());

        Config {
            project_root,
            skills_dir,
            data_dir,
            bind_addr,
            openai_api_key,
            openai_tts_model,
            openai_tts_voice,
        }
    }
}
