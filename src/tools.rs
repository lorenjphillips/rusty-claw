use once_cell::sync::Lazy;

use crate::models::{PortingBacklog, PortingModule};
use crate::permissions::ToolPermissionContext;

static RAW_JSON: &str = include_str!("../reference_data/tools_snapshot.json");

pub static PORTED_TOOLS: Lazy<Vec<PortingModule>> = Lazy::new(|| {
    let raw: Vec<PortingModule> = serde_json::from_str(RAW_JSON).expect("tools JSON");
    raw.into_iter()
        .map(|mut m| {
            m.status = "mirrored".into();
            m
        })
        .collect()
});

#[derive(Debug, Clone)]
pub struct ToolExecution {
    pub name: String,
    pub source_hint: String,
    pub payload: String,
    pub handled: bool,
    pub message: String,
}

pub fn build_tool_backlog() -> PortingBacklog {
    PortingBacklog {
        title: "Tool surface".into(),
        modules: PORTED_TOOLS.clone(),
    }
}

pub fn get_tool(name: &str) -> Option<&PortingModule> {
    let needle = name.to_lowercase();
    PORTED_TOOLS
        .iter()
        .find(|m| m.name.to_lowercase() == needle)
}

pub fn filter_tools_by_permission(
    tools: Vec<PortingModule>,
    ctx: Option<&ToolPermissionContext>,
) -> Vec<PortingModule> {
    match ctx {
        Some(ctx) => tools.into_iter().filter(|m| !ctx.blocks(&m.name)).collect(),
        None => tools,
    }
}

pub fn get_tools(
    simple_mode: bool,
    include_mcp: bool,
    permission_context: Option<&ToolPermissionContext>,
) -> Vec<PortingModule> {
    let mut tools: Vec<PortingModule> = PORTED_TOOLS.clone();
    if simple_mode {
        tools.retain(|m| matches!(m.name.as_str(), "BashTool" | "FileReadTool" | "FileEditTool"));
    }
    if !include_mcp {
        tools.retain(|m| {
            !m.name.to_lowercase().contains("mcp")
                && !m.source_hint.to_lowercase().contains("mcp")
        });
    }
    filter_tools_by_permission(tools, permission_context)
}

pub fn find_tools(query: &str, limit: usize) -> Vec<PortingModule> {
    let needle = query.to_lowercase();
    PORTED_TOOLS
        .iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&needle)
                || m.source_hint.to_lowercase().contains(&needle)
        })
        .take(limit)
        .cloned()
        .collect()
}

pub fn execute_tool(name: &str, payload: &str) -> ToolExecution {
    match get_tool(name) {
        Some(m) => ToolExecution {
            name: m.name.clone(),
            source_hint: m.source_hint.clone(),
            payload: payload.into(),
            handled: true,
            message: format!(
                "Mirrored tool '{}' from {} would handle payload {:?}.",
                m.name, m.source_hint, payload
            ),
        },
        None => ToolExecution {
            name: name.into(),
            source_hint: String::new(),
            payload: payload.into(),
            handled: false,
            message: format!("Unknown mirrored tool: {}", name),
        },
    }
}

pub fn render_tool_index(limit: usize, query: Option<&str>) -> String {
    let modules: Vec<PortingModule> = match query {
        Some(q) => find_tools(q, limit),
        None => PORTED_TOOLS.iter().take(limit).cloned().collect(),
    };
    let mut lines = vec![format!("Tool entries: {}", PORTED_TOOLS.len()), String::new()];
    if let Some(q) = query {
        lines.push(format!("Filtered by: {}", q));
        lines.push(String::new());
    }
    for m in &modules {
        lines.push(format!("- {} — {}", m.name, m.source_hint));
    }
    lines.join("\n")
}
