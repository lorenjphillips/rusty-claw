use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryLog {
    pub events: Vec<HistoryEvent>,
}

impl HistoryLog {
    pub fn add(&mut self, title: &str, detail: &str) {
        self.events.push(HistoryEvent {
            title: title.into(),
            detail: detail.into(),
        });
    }

    pub fn as_markdown(&self) -> String {
        let mut lines = vec!["# Session History".into(), String::new()];
        for e in &self.events {
            lines.push(format!("- {}: {}", e.title, e.detail));
        }
        lines.join("\n")
    }
}
