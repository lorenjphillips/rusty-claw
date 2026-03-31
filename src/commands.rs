use once_cell::sync::Lazy;

use crate::models::{PortingBacklog, PortingModule};

static RAW_JSON: &str = include_str!("../reference_data/commands_snapshot.json");

pub static PORTED_COMMANDS: Lazy<Vec<PortingModule>> = Lazy::new(|| {
    let raw: Vec<PortingModule> = serde_json::from_str(RAW_JSON).expect("commands JSON");
    raw.into_iter()
        .map(|mut m| {
            m.status = "mirrored".into();
            m
        })
        .collect()
});

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub name: String,
    pub source_hint: String,
    pub prompt: String,
    pub handled: bool,
    pub message: String,
}

pub fn built_in_command_names() -> Vec<String> {
    PORTED_COMMANDS.iter().map(|m| m.name.clone()).collect()
}

pub fn build_command_backlog() -> PortingBacklog {
    PortingBacklog {
        title: "Command surface".into(),
        modules: PORTED_COMMANDS.clone(),
    }
}

pub fn get_command(name: &str) -> Option<&PortingModule> {
    let needle = name.to_lowercase();
    PORTED_COMMANDS
        .iter()
        .find(|m| m.name.to_lowercase() == needle)
}

pub fn get_commands(
    include_plugin_commands: bool,
    include_skill_commands: bool,
) -> Vec<PortingModule> {
    PORTED_COMMANDS
        .iter()
        .filter(|m| include_plugin_commands || !m.source_hint.to_lowercase().contains("plugin"))
        .filter(|m| include_skill_commands || !m.source_hint.to_lowercase().contains("skills"))
        .cloned()
        .collect()
}

pub fn find_commands(query: &str, limit: usize) -> Vec<PortingModule> {
    let needle = query.to_lowercase();
    PORTED_COMMANDS
        .iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&needle)
                || m.source_hint.to_lowercase().contains(&needle)
        })
        .take(limit)
        .cloned()
        .collect()
}

pub fn execute_command(name: &str, prompt: &str) -> CommandExecution {
    match get_command(name) {
        Some(m) => CommandExecution {
            name: m.name.clone(),
            source_hint: m.source_hint.clone(),
            prompt: prompt.into(),
            handled: true,
            message: format!(
                "Mirrored command '{}' from {} would handle prompt {:?}.",
                m.name, m.source_hint, prompt
            ),
        },
        None => CommandExecution {
            name: name.into(),
            source_hint: String::new(),
            prompt: prompt.into(),
            handled: false,
            message: format!("Unknown mirrored command: {}", name),
        },
    }
}

pub fn render_command_index(limit: usize, query: Option<&str>) -> String {
    let modules: Vec<PortingModule> = match query {
        Some(q) => find_commands(q, limit),
        None => PORTED_COMMANDS.iter().take(limit).cloned().collect(),
    };
    let mut lines = vec![format!("Command entries: {}", PORTED_COMMANDS.len()), String::new()];
    if let Some(q) = query {
        lines.push(format!("Filtered by: {}", q));
        lines.push(String::new());
    }
    for m in &modules {
        lines.push(format!("- {} — {}", m.name, m.source_hint));
    }
    lines.join("\n")
}
