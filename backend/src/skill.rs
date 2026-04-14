use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A map layer declared by a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayer {
    pub id: String,
    #[serde(rename = "type")]
    pub layer_type: String,
}

/// YAML front matter from a skill.md file.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontMatter {
    pub skill: String,
    pub source_type: Option<String>,
    pub source_path: Option<String>,
    pub interactions: Option<Vec<String>>,
    pub map_layers: Option<Vec<MapLayer>>,
}

/// A tool declared in the ## Tools section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
    pub sql_file: Option<String>,
}

/// A parameter for a skill tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
}

/// Fully parsed skill manifest.
#[derive(Debug, Clone, Serialize)]
pub struct SkillManifest {
    pub name: String,
    pub source_type: Option<String>,
    pub source_path: Option<String>,
    pub interactions: Vec<String>,
    pub map_layers: Vec<MapLayer>,
    pub tools: Vec<SkillTool>,
    pub domain_prompt: String,
    pub sql_views: String,
    /// Directory containing SQL files for this skill
    #[serde(skip)]
    #[allow(dead_code)]
    pub sql_dir: PathBuf,
    /// All SQL files keyed by filename (without extension)
    pub sql_files: HashMap<String, String>,
}

/// Load all skills from the skills/ directory.
pub fn load_skills(skills_dir: &Path) -> Vec<SkillManifest> {
    let mut skills = Vec::new();

    let pattern = skills_dir.join("*/skill.md");
    let pattern_str = pattern.to_string_lossy().to_string();

    for entry in glob::glob(&pattern_str).expect("invalid skill glob pattern") {
        match entry {
            Ok(path) => match parse_skill(&path) {
                Ok(skill) => {
                    tracing::info!(skill = %skill.name, "loaded skill");
                    skills.push(skill);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to parse skill");
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "glob error scanning skills");
            }
        }
    }

    skills
}

/// Parse a single skill.md file into a SkillManifest.
fn parse_skill(path: &Path) -> Result<SkillManifest, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read error: {e}"))?;

    // Split YAML front matter from markdown body
    let (front_matter_str, body) = split_front_matter(&content)?;

    let fm: SkillFrontMatter =
        serde_yaml::from_str(&front_matter_str).map_err(|e| format!("YAML parse error: {e}"))?;

    // Parse markdown sections
    let sections = parse_sections(&body);

    // Extract tools from ## Tools section
    let tools = sections
        .get("Tools")
        .map(|s| parse_tools(s))
        .unwrap_or_default();

    // Extract domain prompt
    let domain_prompt = sections.get("Domain prompt").cloned().unwrap_or_default();

    // Load SQL files from the skill's sql/ directory
    let skill_dir = path.parent().unwrap();
    let sql_dir = skill_dir.join("sql");
    let mut sql_files = HashMap::new();
    let mut sql_views = String::new();

    if sql_dir.is_dir() {
        let sql_pattern = sql_dir.join("*.sql");
        let sql_pattern_str = sql_pattern.to_string_lossy().to_string();

        for entry in glob::glob(&sql_pattern_str).unwrap_or_else(|_| panic!("bad sql glob")) {
            if let Ok(sql_path) = entry {
                if let Ok(sql_content) = fs::read_to_string(&sql_path) {
                    let stem = sql_path.file_stem().unwrap().to_string_lossy().to_string();
                    if stem == "views" {
                        sql_views = sql_content.clone();
                    }
                    sql_files.insert(stem, sql_content);
                }
            }
        }
    }

    Ok(SkillManifest {
        name: fm.skill,
        source_type: fm.source_type,
        source_path: fm.source_path,
        interactions: fm.interactions.unwrap_or_default(),
        map_layers: fm.map_layers.unwrap_or_default(),
        tools,
        domain_prompt,
        sql_views,
        sql_dir,
        sql_files,
    })
}

/// Split `---\n...\n---` YAML front matter from the markdown body.
fn split_front_matter(content: &str) -> Result<(String, String), String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("no YAML front matter found".into());
    }

    let after_first = &trimmed[3..];
    let end = after_first
        .find("\n---")
        .ok_or("unterminated front matter")?;

    let yaml = after_first[..end].trim().to_string();
    let body = after_first[end + 4..].to_string();

    Ok((yaml, body))
}

/// Parse markdown into sections keyed by ## heading.
fn parse_sections(body: &str) -> HashMap<String, String> {
    let mut sections = HashMap::new();
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();

    for line in body.lines() {
        if line.starts_with("## ") {
            if let Some(heading) = current_heading.take() {
                sections.insert(heading, current_content.trim().to_string());
            }
            current_heading = Some(line[3..].trim().to_string());
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if let Some(heading) = current_heading {
        sections.insert(heading, current_content.trim().to_string());
    }

    sections
}

/// Parse the ## Tools section into SkillTool entries.
fn parse_tools(tools_text: &str) -> Vec<SkillTool> {
    let mut tools = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_desc = String::new();
    let mut current_params: Vec<ToolParam> = Vec::new();
    let mut current_sql: Option<String> = None;

    for line in tools_text.lines() {
        if line.starts_with("### ") {
            // Flush previous tool
            if let Some(name) = current_name.take() {
                tools.push(SkillTool {
                    name,
                    description: current_desc.trim().to_string(),
                    parameters: std::mem::take(&mut current_params),
                    sql_file: current_sql.take(),
                });
            }
            current_name = Some(line[4..].trim().to_string());
            current_desc.clear();
            current_params.clear();
            current_sql = None;
        } else if line.starts_with("- Parameters:") {
            // Parse parameter list: `mmsi` (integer), `limit` (integer, default 100)
            let params_str = &line["- Parameters:".len()..];
            current_params = parse_param_list(params_str);
        } else if line.starts_with("- SQL:") {
            current_sql = Some(line["- SQL:".len()..].trim().trim_matches('`').to_string());
        } else if current_name.is_some() && !line.trim().is_empty() && !line.starts_with("- ") {
            current_desc.push_str(line.trim());
            current_desc.push(' ');
        }
    }

    // Flush last tool
    if let Some(name) = current_name {
        tools.push(SkillTool {
            name,
            description: current_desc.trim().to_string(),
            parameters: current_params,
            sql_file: current_sql,
        });
    }

    tools
}

/// Parse parameter string like: `mmsi` (integer), `limit` (integer, default 100)
fn parse_param_list(text: &str) -> Vec<ToolParam> {
    let mut params = Vec::new();

    // Split on `, ` but be careful of commas inside parens
    for segment in text.split("), ") {
        let segment = segment.trim().trim_end_matches(')');
        // Expected: `name` (type[, optional][, default X])
        if let Some(backtick_start) = segment.find('`') {
            let after_first = &segment[backtick_start + 1..];
            if let Some(backtick_end) = after_first.find('`') {
                let name = after_first[..backtick_end].to_string();
                let rest = &after_first[backtick_end + 1..];

                // Extract type and modifiers from parenthesized part
                let paren_content = rest
                    .trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim();

                let parts: Vec<&str> = paren_content.split(',').map(|s| s.trim()).collect();
                let param_type = parts.first().unwrap_or(&"string").to_string();
                let is_optional = parts.iter().any(|p| *p == "optional");
                let default_val = parts
                    .iter()
                    .find_map(|p| p.strip_prefix("default ").map(|v| v.to_string()));

                params.push(ToolParam {
                    name,
                    param_type,
                    required: !is_optional && default_val.is_none(),
                    default: default_val,
                });
            }
        }
    }

    params
}
