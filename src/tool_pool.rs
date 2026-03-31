use crate::models::PortingModule;
use crate::permissions::ToolPermissionContext;
use crate::tools::get_tools;

pub struct ToolPool {
    pub tools: Vec<PortingModule>,
    pub simple_mode: bool,
    pub include_mcp: bool,
}

impl ToolPool {
    pub fn as_markdown(&self) -> String {
        let mut lines = vec![
            "# Tool Pool".into(),
            String::new(),
            format!("Simple mode: {}", self.simple_mode),
            format!("Include MCP: {}", self.include_mcp),
            format!("Tool count: {}", self.tools.len()),
        ];
        for tool in self.tools.iter().take(15) {
            lines.push(format!("- {} — {}", tool.name, tool.source_hint));
        }
        lines.join("\n")
    }
}

pub fn assemble_tool_pool(
    simple_mode: bool,
    include_mcp: bool,
    permission_context: Option<&ToolPermissionContext>,
) -> ToolPool {
    ToolPool {
        tools: get_tools(simple_mode, include_mcp, permission_context),
        simple_mode,
        include_mcp,
    }
}
