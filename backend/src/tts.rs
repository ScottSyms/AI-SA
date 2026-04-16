use serde::Serialize;

const OPENAI_AUDIO_SPEECH_URL: &str = "https://api.openai.com/v1/audio/speech";

#[derive(Clone)]
pub struct OpenAiTtsClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
    voice: String,
}

#[derive(Serialize)]
struct SpeechRequest<'a> {
    model: &'a str,
    voice: &'a str,
    input: &'a str,
    response_format: &'a str,
}

impl OpenAiTtsClient {
    pub fn new(api_key: String, model: String, voice: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            model,
            voice,
        }
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        let request = SpeechRequest {
            model: &self.model,
            voice: &self.voice,
            input: text,
            response_format: "mp3",
        };

        let resp = self
            .http
            .post(OPENAI_AUDIO_SPEECH_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?;

        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.bytes().await.map_err(|e| format!("body read error: {e}"))?;

        if !status.is_success() {
            let message = String::from_utf8_lossy(&body).to_string();
            return Err(format!("OpenAI TTS error ({status}): {message}"));
        }

        if let Some(content_type) = headers.get(reqwest::header::CONTENT_TYPE) {
            tracing::debug!(content_type = ?content_type, bytes = body.len(), "received TTS audio");
        }

        Ok(body.to_vec())
    }
}
