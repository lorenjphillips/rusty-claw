use crate::commands::get_commands;
use crate::models::PortingModule;

pub struct CommandGraph {
    pub builtins: Vec<PortingModule>,
    pub plugin_like: Vec<PortingModule>,
    pub skill_like: Vec<PortingModule>,
}

impl CommandGraph {
    pub fn as_markdown(&self) -> String {
        [
            "# Command Graph",
            "",
            &format!("Builtins: {}", self.builtins.len()),
            &format!("Plugin-like commands: {}", self.plugin_like.len()),
            &format!("Skill-like commands: {}", self.skill_like.len()),
        ]
        .join("\n")
    }
}

pub fn build_command_graph() -> CommandGraph {
    let commands = get_commands(true, true);
    let builtins: Vec<PortingModule> = commands
        .iter()
        .filter(|m| {
            !m.source_hint.to_lowercase().contains("plugin")
                && !m.source_hint.to_lowercase().contains("skills")
        })
        .cloned()
        .collect();
    let plugin_like: Vec<PortingModule> = commands
        .iter()
        .filter(|m| m.source_hint.to_lowercase().contains("plugin"))
        .cloned()
        .collect();
    let skill_like: Vec<PortingModule> = commands
        .iter()
        .filter(|m| m.source_hint.to_lowercase().contains("skills"))
        .cloned()
        .collect();
    CommandGraph {
        builtins,
        plugin_like,
        skill_like,
    }
}
