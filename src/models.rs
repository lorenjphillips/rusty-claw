use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsystem {
    pub name: String,
    pub path: String,
    pub file_count: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortingModule {
    pub name: String,
    pub responsibility: String,
    pub source_hint: String,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "planned".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDenial {
    pub tool_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSummary {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl UsageSummary {
    pub fn add_turn(&self, prompt: &str, output: &str) -> Self {
        Self {
            input_tokens: self.input_tokens + prompt.split_whitespace().count(),
            output_tokens: self.output_tokens + output.split_whitespace().count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortingBacklog {
    pub title: String,
    pub modules: Vec<PortingModule>,
}

impl PortingBacklog {
    pub fn summary_lines(&self) -> Vec<String> {
        self.modules
            .iter()
            .map(|m| {
                format!(
                    "- {} [{}] — {} (from {})",
                    m.name, m.status, m.responsibility, m.source_hint
                )
            })
            .collect()
    }
}
