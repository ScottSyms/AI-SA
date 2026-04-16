use regex::Regex;
use std::sync::OnceLock;

fn mmsi_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bMMSI\s*[:=]?\s*(\d{6,12})\b").unwrap())
}

fn spell_digits(value: &str) -> String {
    value
        .chars()
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_speech_text(text: &str) -> String {
    let cleaned = text.replace("**", "").replace('*', "").replace("\n", ". ");

    mmsi_regex()
        .replace_all(&cleaned, |caps: &regex::Captures| {
            format!("MMSI {}", spell_digits(&caps[1]))
        })
        .to_string()
}
