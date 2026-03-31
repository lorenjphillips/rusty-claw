use crate::commands::PORTED_COMMANDS;
use crate::context::{build_port_context, render_context, PortContext};
use crate::execution_registry::build_execution_registry;
use crate::history::HistoryLog;
use crate::models::{PermissionDenial, PortingModule};
use crate::query_engine::{QueryEngineConfig, QueryEnginePort, TurnResult};
use crate::setup::{run_setup, SetupReport, WorkspaceSetup};
use crate::system_init::build_system_init_message;
use crate::tools::PORTED_TOOLS;

#[derive(Debug, Clone)]
pub struct RoutedMatch {
    pub kind: String,
    pub name: String,
    pub source_hint: String,
    pub score: usize,
}

pub struct RuntimeSession {
    pub prompt: String,
    pub context: PortContext,
    pub setup: WorkspaceSetup,
    pub setup_report: SetupReport,
    pub system_init_message: String,
    pub history: HistoryLog,
    pub routed_matches: Vec<RoutedMatch>,
    pub turn_result: TurnResult,
    pub command_execution_messages: Vec<String>,
    pub tool_execution_messages: Vec<String>,
    pub stream_events: Vec<serde_json::Value>,
    pub persisted_session_path: String,
}

impl RuntimeSession {
    pub fn as_markdown(&self) -> String {
        let mut lines = vec![
            "# Runtime Session".into(),
            String::new(),
            format!("Prompt: {}", self.prompt),
            String::new(),
            "## Context".into(),
            render_context(&self.context),
            String::new(),
            "## Setup".into(),
            format!(
                "- Rust version: {} ({})",
                self.setup.rust_version, self.setup.implementation
            ),
            format!("- Platform: {}", self.setup.platform_name),
            format!("- Test command: {}", self.setup.test_command),
            String::new(),
            "## Startup Steps".into(),
        ];
        for step in self.setup.startup_steps() {
            lines.push(format!("- {}", step));
        }
        lines.push(String::new());
        lines.push("## System Init".into());
        lines.push(self.system_init_message.clone());
        lines.push(String::new());
        lines.push("## Routed Matches".into());
        if self.routed_matches.is_empty() {
            lines.push("- none".into());
        } else {
            for m in &self.routed_matches {
                lines.push(format!(
                    "- [{}] {} ({}) — {}",
                    m.kind, m.name, m.score, m.source_hint
                ));
            }
        }
        lines.push(String::new());
        lines.push("## Command Execution".into());
        if self.command_execution_messages.is_empty() {
            lines.push("none".into());
        } else {
            lines.extend(self.command_execution_messages.clone());
        }
        lines.push(String::new());
        lines.push("## Tool Execution".into());
        if self.tool_execution_messages.is_empty() {
            lines.push("none".into());
        } else {
            lines.extend(self.tool_execution_messages.clone());
        }
        lines.push(String::new());
        lines.push("## Stream Events".into());
        for event in &self.stream_events {
            let typ = event.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            lines.push(format!("- {}: {}", typ, event));
        }
        lines.push(String::new());
        lines.push("## Turn Result".into());
        lines.push(self.turn_result.output.clone());
        lines.push(String::new());
        lines.push(format!(
            "Persisted session path: {}",
            self.persisted_session_path
        ));
        lines.push(String::new());
        lines.push(self.history.as_markdown());
        lines.join("\n")
    }
}

pub struct PortRuntime;

impl PortRuntime {
    pub fn route_prompt(&self, prompt: &str, limit: usize) -> Vec<RoutedMatch> {
        let tokens: std::collections::HashSet<String> = prompt
            .replace('/', " ")
            .replace('-', " ")
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .collect();

        let mut by_command = collect_matches(&tokens, &PORTED_COMMANDS, "command");
        let mut by_tool = collect_matches(&tokens, &PORTED_TOOLS, "tool");

        let mut selected: Vec<RoutedMatch> = Vec::new();
        if !by_command.is_empty() {
            selected.push(by_command.remove(0));
        }
        if !by_tool.is_empty() {
            selected.push(by_tool.remove(0));
        }

        let mut leftovers: Vec<RoutedMatch> = by_command.into_iter().chain(by_tool).collect();
        leftovers.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.kind.cmp(&b.kind))
                .then_with(|| a.name.cmp(&b.name))
        });

        let remaining = limit.saturating_sub(selected.len());
        selected.extend(leftovers.into_iter().take(remaining));
        selected.truncate(limit);
        selected
    }

    pub fn bootstrap_session(&self, prompt: &str, limit: usize) -> RuntimeSession {
        let context = build_port_context(None);
        let setup_report = run_setup(None, true);
        let setup = setup_report.setup.clone();
        let mut history = HistoryLog::default();
        let mut engine = QueryEnginePort::from_workspace();

        history.add(
            "context",
            &format!(
                "source_files={}, archive_available={}",
                context.python_file_count, context.archive_available
            ),
        );
        history.add(
            "registry",
            &format!(
                "commands={}, tools={}",
                PORTED_COMMANDS.len(),
                PORTED_TOOLS.len()
            ),
        );

        let matches = self.route_prompt(prompt, limit);
        let registry = build_execution_registry();

        let command_execs: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "command")
            .filter_map(|m| registry.command(&m.name).map(|c| c.execute(prompt)))
            .collect();
        let tool_execs: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "tool")
            .filter_map(|m| registry.tool(&m.name).map(|t| t.execute(prompt)))
            .collect();

        let denials = infer_permission_denials(&matches);
        let matched_commands: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "command")
            .map(|m| m.name.clone())
            .collect();
        let matched_tools: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "tool")
            .map(|m| m.name.clone())
            .collect();

        let stream_events =
            engine.stream_submit_message(prompt, &matched_commands, &matched_tools, &denials);
        let turn_result =
            engine.submit_message(prompt, &matched_commands, &matched_tools, &denials);
        let persisted_session_path = engine.persist_session();

        history.add(
            "routing",
            &format!("matches={} for prompt={:?}", matches.len(), prompt),
        );
        history.add(
            "execution",
            &format!(
                "command_execs={} tool_execs={}",
                command_execs.len(),
                tool_execs.len()
            ),
        );
        history.add(
            "turn",
            &format!(
                "commands={} tools={} denials={} stop={}",
                turn_result.matched_commands.len(),
                turn_result.matched_tools.len(),
                turn_result.permission_denials.len(),
                turn_result.stop_reason
            ),
        );
        history.add("session_store", &persisted_session_path);

        RuntimeSession {
            prompt: prompt.into(),
            context,
            setup,
            setup_report,
            system_init_message: build_system_init_message(true),
            history,
            routed_matches: matches,
            turn_result,
            command_execution_messages: command_execs,
            tool_execution_messages: tool_execs,
            stream_events,
            persisted_session_path,
        }
    }

    pub fn run_turn_loop(
        &self,
        prompt: &str,
        limit: usize,
        max_turns: usize,
        structured_output: bool,
    ) -> Vec<TurnResult> {
        let mut engine = QueryEnginePort::from_workspace();
        engine.config = QueryEngineConfig {
            max_turns,
            structured_output,
            ..QueryEngineConfig::default()
        };
        let matches = self.route_prompt(prompt, limit);
        let command_names: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "command")
            .map(|m| m.name.clone())
            .collect();
        let tool_names: Vec<String> = matches
            .iter()
            .filter(|m| m.kind == "tool")
            .map(|m| m.name.clone())
            .collect();

        let mut results = Vec::new();
        for turn in 0..max_turns {
            let turn_prompt = if turn == 0 {
                prompt.to_string()
            } else {
                format!("{} [turn {}]", prompt, turn + 1)
            };
            let result = engine.submit_message(&turn_prompt, &command_names, &tool_names, &[]);
            let stop = result.stop_reason.clone();
            results.push(result);
            if stop != "completed" {
                break;
            }
        }
        results
    }
}

fn infer_permission_denials(matches: &[RoutedMatch]) -> Vec<PermissionDenial> {
    matches
        .iter()
        .filter(|m| m.kind == "tool" && m.name.to_lowercase().contains("bash"))
        .map(|m| PermissionDenial {
            tool_name: m.name.clone(),
            reason: "destructive shell execution remains gated in the Rust port".into(),
        })
        .collect()
}

fn score(tokens: &std::collections::HashSet<String>, module: &PortingModule) -> usize {
    let haystacks = [
        module.name.to_lowercase(),
        module.source_hint.to_lowercase(),
        module.responsibility.to_lowercase(),
    ];
    tokens
        .iter()
        .filter(|token| haystacks.iter().any(|h| h.contains(token.as_str())))
        .count()
}

fn collect_matches(
    tokens: &std::collections::HashSet<String>,
    modules: &[PortingModule],
    kind: &str,
) -> Vec<RoutedMatch> {
    let mut matches: Vec<RoutedMatch> = modules
        .iter()
        .filter_map(|m| {
            let s = score(tokens, m);
            if s > 0 {
                Some(RoutedMatch {
                    kind: kind.into(),
                    name: m.name.clone(),
                    source_hint: m.source_hint.clone(),
                    score: s,
                })
            } else {
                None
            }
        })
        .collect();
    matches.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    matches
}
