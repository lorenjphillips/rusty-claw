use crate::commands::build_command_backlog;
use crate::models::{PermissionDenial, UsageSummary};
use crate::port_manifest::{build_port_manifest, PortManifest};
use crate::session_store::{load_session, save_session, StoredSession};
use crate::tools::build_tool_backlog;
use crate::transcript::TranscriptStore;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct QueryEngineConfig {
    pub max_turns: usize,
    pub max_budget_tokens: usize,
    pub compact_after_turns: usize,
    pub structured_output: bool,
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self {
            max_turns: 8,
            max_budget_tokens: 2000,
            compact_after_turns: 12,
            structured_output: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurnResult {
    pub prompt: String,
    pub output: String,
    pub matched_commands: Vec<String>,
    pub matched_tools: Vec<String>,
    pub permission_denials: Vec<PermissionDenial>,
    pub usage: UsageSummary,
    pub stop_reason: String,
}

pub struct QueryEnginePort {
    pub manifest: PortManifest,
    pub config: QueryEngineConfig,
    pub session_id: String,
    pub mutable_messages: Vec<String>,
    pub permission_denials: Vec<PermissionDenial>,
    pub total_usage: UsageSummary,
    pub transcript_store: TranscriptStore,
}

impl QueryEnginePort {
    pub fn new(manifest: PortManifest) -> Self {
        Self {
            manifest,
            config: QueryEngineConfig::default(),
            session_id: uuid::Uuid::new_v4().to_string().replace('-', ""),
            mutable_messages: Vec::new(),
            permission_denials: Vec::new(),
            total_usage: UsageSummary::default(),
            transcript_store: TranscriptStore::default(),
        }
    }

    pub fn from_workspace() -> Self {
        Self::new(build_port_manifest(None))
    }

    pub fn from_saved_session(session_id: &str) -> Self {
        let stored = load_session(session_id, None);
        let transcript = TranscriptStore {
            entries: stored.messages.clone(),
            flushed: true,
        };
        Self {
            manifest: build_port_manifest(None),
            config: QueryEngineConfig::default(),
            session_id: stored.session_id,
            mutable_messages: stored.messages,
            permission_denials: Vec::new(),
            total_usage: UsageSummary {
                input_tokens: stored.input_tokens,
                output_tokens: stored.output_tokens,
            },
            transcript_store: transcript,
        }
    }

    pub fn submit_message(
        &mut self,
        prompt: &str,
        matched_commands: &[String],
        matched_tools: &[String],
        denied_tools: &[PermissionDenial],
    ) -> TurnResult {
        if self.mutable_messages.len() >= self.config.max_turns {
            let output = format!("Max turns reached before processing prompt: {}", prompt);
            return TurnResult {
                prompt: prompt.into(),
                output,
                matched_commands: matched_commands.to_vec(),
                matched_tools: matched_tools.to_vec(),
                permission_denials: denied_tools.to_vec(),
                usage: self.total_usage.clone(),
                stop_reason: "max_turns_reached".into(),
            };
        }

        let cmd_str = if matched_commands.is_empty() {
            "none".into()
        } else {
            matched_commands.join(", ")
        };
        let tool_str = if matched_tools.is_empty() {
            "none".into()
        } else {
            matched_tools.join(", ")
        };
        let summary_lines = vec![
            format!("Prompt: {}", prompt),
            format!("Matched commands: {}", cmd_str),
            format!("Matched tools: {}", tool_str),
            format!("Permission denials: {}", denied_tools.len()),
        ];
        let output = self.format_output(&summary_lines);
        let projected_usage = self.total_usage.add_turn(prompt, &output);
        let stop_reason =
            if projected_usage.input_tokens + projected_usage.output_tokens > self.config.max_budget_tokens {
                "max_budget_reached"
            } else {
                "completed"
            };

        self.mutable_messages.push(prompt.into());
        self.transcript_store.append(prompt.into());
        self.permission_denials.extend(denied_tools.iter().cloned());
        self.total_usage = projected_usage.clone();
        self.compact_messages_if_needed();

        TurnResult {
            prompt: prompt.into(),
            output,
            matched_commands: matched_commands.to_vec(),
            matched_tools: matched_tools.to_vec(),
            permission_denials: denied_tools.to_vec(),
            usage: projected_usage,
            stop_reason: stop_reason.into(),
        }
    }

    pub fn stream_submit_message(
        &mut self,
        prompt: &str,
        matched_commands: &[String],
        matched_tools: &[String],
        denied_tools: &[PermissionDenial],
    ) -> Vec<serde_json::Value> {
        let mut events = Vec::new();
        events.push(json!({
            "type": "message_start",
            "session_id": self.session_id,
            "prompt": prompt
        }));
        if !matched_commands.is_empty() {
            events.push(json!({"type": "command_match", "commands": matched_commands}));
        }
        if !matched_tools.is_empty() {
            events.push(json!({"type": "tool_match", "tools": matched_tools}));
        }
        if !denied_tools.is_empty() {
            let names: Vec<&str> = denied_tools.iter().map(|d| d.tool_name.as_str()).collect();
            events.push(json!({"type": "permission_denial", "denials": names}));
        }
        let result = self.submit_message(prompt, matched_commands, matched_tools, denied_tools);
        events.push(json!({"type": "message_delta", "text": result.output}));
        events.push(json!({
            "type": "message_stop",
            "usage": {"input_tokens": result.usage.input_tokens, "output_tokens": result.usage.output_tokens},
            "stop_reason": result.stop_reason,
            "transcript_size": self.transcript_store.entries.len()
        }));
        events
    }

    fn compact_messages_if_needed(&mut self) {
        if self.mutable_messages.len() > self.config.compact_after_turns {
            let start = self.mutable_messages.len() - self.config.compact_after_turns;
            self.mutable_messages = self.mutable_messages[start..].to_vec();
        }
        self.transcript_store.compact(self.config.compact_after_turns);
    }

    pub fn persist_session(&mut self) -> String {
        self.transcript_store.flush();
        let stored = StoredSession {
            session_id: self.session_id.clone(),
            messages: self.mutable_messages.clone(),
            input_tokens: self.total_usage.input_tokens,
            output_tokens: self.total_usage.output_tokens,
        };
        let path = save_session(&stored, None);
        path.display().to_string()
    }

    fn format_output(&self, summary_lines: &[String]) -> String {
        if self.config.structured_output {
            let payload = json!({
                "summary": summary_lines,
                "session_id": self.session_id
            });
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| {
                serde_json::to_string_pretty(&json!({
                    "summary": ["structured output retry"],
                    "session_id": self.session_id
                }))
                .unwrap()
            })
        } else {
            summary_lines.join("\n")
        }
    }

    pub fn render_summary(&self) -> String {
        let command_backlog = build_command_backlog();
        let tool_backlog = build_tool_backlog();
        let cmd_lines = command_backlog.summary_lines();
        let tool_lines = tool_backlog.summary_lines();
        let mut sections = vec![
            "# Python Porting Workspace Summary".into(),
            String::new(),
            self.manifest.to_markdown(),
            String::new(),
            format!(
                "Command surface: {} mirrored entries",
                command_backlog.modules.len()
            ),
        ];
        sections.extend(cmd_lines.into_iter().take(10));
        sections.push(String::new());
        sections.push(format!(
            "Tool surface: {} mirrored entries",
            tool_backlog.modules.len()
        ));
        sections.extend(tool_lines.into_iter().take(10));
        sections.push(String::new());
        sections.push(format!("Session id: {}", self.session_id));
        sections.push(format!(
            "Conversation turns stored: {}",
            self.mutable_messages.len()
        ));
        sections.push(format!(
            "Permission denials tracked: {}",
            self.permission_denials.len()
        ));
        sections.push(format!(
            "Usage totals: in={} out={}",
            self.total_usage.input_tokens, self.total_usage.output_tokens
        ));
        sections.push(format!("Max turns: {}", self.config.max_turns));
        sections.push(format!(
            "Max budget tokens: {}",
            self.config.max_budget_tokens
        ));
        sections.push(format!(
            "Transcript flushed: {}",
            self.transcript_store.flushed
        ));
        sections.join("\n")
    }
}
