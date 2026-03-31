use crate::commands::{execute_command, PORTED_COMMANDS};
use crate::tools::{execute_tool, PORTED_TOOLS};

#[derive(Debug, Clone)]
pub struct MirroredCommand {
    pub name: String,
    pub source_hint: String,
}

impl MirroredCommand {
    pub fn execute(&self, prompt: &str) -> String {
        execute_command(&self.name, prompt).message
    }
}

#[derive(Debug, Clone)]
pub struct MirroredTool {
    pub name: String,
    pub source_hint: String,
}

impl MirroredTool {
    pub fn execute(&self, payload: &str) -> String {
        execute_tool(&self.name, payload).message
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionRegistry {
    pub commands: Vec<MirroredCommand>,
    pub tools: Vec<MirroredTool>,
}

impl ExecutionRegistry {
    pub fn command(&self, name: &str) -> Option<&MirroredCommand> {
        let lowered = name.to_lowercase();
        self.commands
            .iter()
            .find(|c| c.name.to_lowercase() == lowered)
    }

    pub fn tool(&self, name: &str) -> Option<&MirroredTool> {
        let lowered = name.to_lowercase();
        self.tools
            .iter()
            .find(|t| t.name.to_lowercase() == lowered)
    }
}

pub fn build_execution_registry() -> ExecutionRegistry {
    ExecutionRegistry {
        commands: PORTED_COMMANDS
            .iter()
            .map(|m| MirroredCommand {
                name: m.name.clone(),
                source_hint: m.source_hint.clone(),
            })
            .collect(),
        tools: PORTED_TOOLS
            .iter()
            .map(|m| MirroredTool {
                name: m.name.clone(),
                source_hint: m.source_hint.clone(),
            })
            .collect(),
    }
}
