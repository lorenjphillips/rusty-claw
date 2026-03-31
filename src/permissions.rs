use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ToolPermissionContext {
    deny_names: HashSet<String>,
    deny_prefixes: Vec<String>,
}

impl ToolPermissionContext {
    pub fn new(deny_names: &[String], deny_prefixes: &[String]) -> Self {
        Self {
            deny_names: deny_names.iter().map(|n| n.to_lowercase()).collect(),
            deny_prefixes: deny_prefixes.iter().map(|p| p.to_lowercase()).collect(),
        }
    }

    pub fn blocks(&self, tool_name: &str) -> bool {
        let lowered = tool_name.to_lowercase();
        self.deny_names.contains(&lowered)
            || self.deny_prefixes.iter().any(|p| lowered.starts_with(p))
    }
}
