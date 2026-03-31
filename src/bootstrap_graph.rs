pub struct BootstrapGraph {
    pub stages: Vec<&'static str>,
}

impl BootstrapGraph {
    pub fn as_markdown(&self) -> String {
        let mut lines = vec!["# Bootstrap Graph".into(), String::new()];
        for stage in &self.stages {
            lines.push(format!("- {}", stage));
        }
        lines.join("\n")
    }
}

pub fn build_bootstrap_graph() -> BootstrapGraph {
    BootstrapGraph {
        stages: vec![
            "top-level prefetch side effects",
            "warning handler and environment guards",
            "CLI parser and pre-action trust gate",
            "setup() + commands/agents parallel load",
            "deferred init after trust",
            "mode routing: local / remote / ssh / teleport / direct-connect / deep-link",
            "query engine submit loop",
        ],
    }
}
