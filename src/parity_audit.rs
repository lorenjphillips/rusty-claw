use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::PORTED_COMMANDS;
use crate::tools::PORTED_TOOLS;

static ARCHIVE_SURFACE_JSON: &str = include_str!("../reference_data/archive_surface_snapshot.json");

static ARCHIVE_ROOT_FILES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    [
        ("QueryEngine.ts", "QueryEngine.py"),
        ("Task.ts", "task.py"),
        ("Tool.ts", "Tool.py"),
        ("commands.ts", "commands.py"),
        ("context.ts", "context.py"),
        ("cost-tracker.ts", "cost_tracker.py"),
        ("costHook.ts", "costHook.py"),
        ("dialogLaunchers.tsx", "dialogLaunchers.py"),
        ("history.ts", "history.py"),
        ("ink.ts", "ink.py"),
        ("interactiveHelpers.tsx", "interactiveHelpers.py"),
        ("main.tsx", "main.py"),
        ("projectOnboardingState.ts", "projectOnboardingState.py"),
        ("query.ts", "query.py"),
        ("replLauncher.tsx", "replLauncher.py"),
        ("setup.ts", "setup.py"),
        ("tasks.ts", "tasks.py"),
        ("tools.ts", "tools.py"),
    ]
    .into_iter()
    .collect()
});

static ARCHIVE_DIR_MAPPINGS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    [
        ("assistant", "assistant"),
        ("bootstrap", "bootstrap"),
        ("bridge", "bridge"),
        ("buddy", "buddy"),
        ("cli", "cli"),
        ("commands", "commands.py"),
        ("components", "components"),
        ("constants", "constants"),
        ("context", "context.py"),
        ("coordinator", "coordinator"),
        ("entrypoints", "entrypoints"),
        ("hooks", "hooks"),
        ("ink", "ink.py"),
        ("keybindings", "keybindings"),
        ("memdir", "memdir"),
        ("migrations", "migrations"),
        ("moreright", "moreright"),
        ("native-ts", "native_ts"),
        ("outputStyles", "outputStyles"),
        ("plugins", "plugins"),
        ("query", "query.py"),
        ("remote", "remote"),
        ("schemas", "schemas"),
        ("screens", "screens"),
        ("server", "server"),
        ("services", "services"),
        ("skills", "skills"),
        ("state", "state"),
        ("tasks", "tasks.py"),
        ("tools", "tools.py"),
        ("types", "types"),
        ("upstreamproxy", "upstreamproxy"),
        ("utils", "utils"),
        ("vim", "vim"),
        ("voice", "voice"),
    ]
    .into_iter()
    .collect()
});

pub struct ParityAuditResult {
    pub archive_present: bool,
    pub root_file_coverage: (usize, usize),
    pub directory_coverage: (usize, usize),
    pub total_file_ratio: (usize, usize),
    pub command_entry_ratio: (usize, usize),
    pub tool_entry_ratio: (usize, usize),
    pub missing_root_targets: Vec<String>,
    pub missing_directory_targets: Vec<String>,
}

impl ParityAuditResult {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec!["# Parity Audit".into()];
        if !self.archive_present {
            lines.push(
                "Local archive unavailable; parity audit cannot compare against the original snapshot.".into(),
            );
            return lines.join("\n");
        }
        lines.push(String::new());
        lines.push(format!(
            "Root file coverage: **{}/{}**",
            self.root_file_coverage.0, self.root_file_coverage.1
        ));
        lines.push(format!(
            "Directory coverage: **{}/{}**",
            self.directory_coverage.0, self.directory_coverage.1
        ));
        lines.push(format!(
            "Total Rust files vs archived TS-like files: **{}/{}**",
            self.total_file_ratio.0, self.total_file_ratio.1
        ));
        lines.push(format!(
            "Command entry coverage: **{}/{}**",
            self.command_entry_ratio.0, self.command_entry_ratio.1
        ));
        lines.push(format!(
            "Tool entry coverage: **{}/{}**",
            self.tool_entry_ratio.0, self.tool_entry_ratio.1
        ));
        lines.push(String::new());
        lines.push("Missing root targets:".into());
        if self.missing_root_targets.is_empty() {
            lines.push("- none".into());
        } else {
            for item in &self.missing_root_targets {
                lines.push(format!("- {}", item));
            }
        }
        lines.push(String::new());
        lines.push("Missing directory targets:".into());
        if self.missing_directory_targets.is_empty() {
            lines.push("- none".into());
        } else {
            for item in &self.missing_directory_targets {
                lines.push(format!("- {}", item));
            }
        }
        lines.join("\n")
    }
}

pub fn run_parity_audit() -> ParityAuditResult {
    let current_root = PathBuf::from("src");
    let archive_root = PathBuf::from("archive")
        .join("claude_code_ts_snapshot")
        .join("src");

    let current_entries: std::collections::HashSet<String> = if current_root.exists() {
        std::fs::read_dir(&current_root)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    let root_hits: Vec<&str> = ARCHIVE_ROOT_FILES
        .values()
        .filter(|target| current_entries.contains(**target))
        .copied()
        .collect();
    let dir_hits: Vec<&str> = ARCHIVE_DIR_MAPPINGS
        .values()
        .filter(|target| current_entries.contains(**target))
        .copied()
        .collect();
    let missing_roots: Vec<String> = ARCHIVE_ROOT_FILES
        .values()
        .filter(|target| !current_entries.contains(**target))
        .map(|s| s.to_string())
        .collect();
    let missing_dirs: Vec<String> = ARCHIVE_DIR_MAPPINGS
        .values()
        .filter(|target| !current_entries.contains(**target))
        .map(|s| s.to_string())
        .collect();

    let current_rs_files = walkdir::WalkDir::new(&current_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "rs")
        })
        .count();

    let reference: serde_json::Value =
        serde_json::from_str(ARCHIVE_SURFACE_JSON).expect("archive surface JSON");
    let total_ts_files = reference["total_ts_like_files"].as_u64().unwrap_or(0) as usize;
    let cmd_entry_count = reference["command_entry_count"].as_u64().unwrap_or(0) as usize;
    let tool_entry_count = reference["tool_entry_count"].as_u64().unwrap_or(0) as usize;

    ParityAuditResult {
        archive_present: archive_root.exists(),
        root_file_coverage: (root_hits.len(), ARCHIVE_ROOT_FILES.len()),
        directory_coverage: (dir_hits.len(), ARCHIVE_DIR_MAPPINGS.len()),
        total_file_ratio: (current_rs_files, total_ts_files),
        command_entry_ratio: (PORTED_COMMANDS.len(), cmd_entry_count),
        tool_entry_ratio: (PORTED_TOOLS.len(), tool_entry_count),
        missing_root_targets: missing_roots,
        missing_directory_targets: missing_dirs,
    }
}
