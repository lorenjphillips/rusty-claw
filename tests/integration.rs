use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rusty-claw").unwrap()
}

#[test]
fn test_summary_runs() {
    cmd()
        .arg("summary")
        .assert()
        .success()
        .stdout(predicate::str::contains("Python Porting Workspace Summary"))
        .stdout(predicate::str::contains("Command surface:"))
        .stdout(predicate::str::contains("Tool surface:"));
}

#[test]
fn test_manifest_runs() {
    cmd()
        .arg("manifest")
        .assert()
        .success()
        .stdout(predicate::str::contains("Port root:"))
        .stdout(predicate::str::contains("Total Rust files:"));
}

#[test]
fn test_parity_audit_runs() {
    cmd()
        .arg("parity-audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parity Audit"));
}

#[test]
fn test_setup_report_runs() {
    cmd()
        .arg("setup-report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup Report"))
        .stdout(predicate::str::contains("Deferred init:"))
        .stdout(predicate::str::contains("plugin_init=true"));
}

#[test]
fn test_command_graph_runs() {
    cmd()
        .arg("command-graph")
        .assert()
        .success()
        .stdout(predicate::str::contains("Command Graph"));
}

#[test]
fn test_tool_pool_runs() {
    cmd()
        .arg("tool-pool")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tool Pool"));
}

#[test]
fn test_bootstrap_graph_runs() {
    cmd()
        .arg("bootstrap-graph")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bootstrap Graph"));
}

#[test]
fn test_subsystems_runs() {
    cmd()
        .args(["subsystems", "--limit", "5"])
        .assert()
        .success();
}

#[test]
fn test_commands_with_query() {
    cmd()
        .args(["commands", "--limit", "5", "--query", "review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Command entries:"));
}

#[test]
fn test_commands_with_filters() {
    cmd()
        .args(["commands", "--limit", "5", "--no-plugin-commands"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Command entries:"));
}

#[test]
fn test_tools_with_query() {
    cmd()
        .args(["tools", "--limit", "5", "--query", "MCP"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tool entries:"));
}

#[test]
fn test_tools_with_filters() {
    cmd()
        .args(["tools", "--limit", "5", "--simple-mode", "--no-mcp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tool entries:"));
}

#[test]
fn test_tool_permission_filtering() {
    cmd()
        .args(["tools", "--limit", "10", "--deny-prefix", "mcp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tool entries:"))
        .stdout(predicate::str::contains("MCPTool").not());
}

#[test]
fn test_route_runs() {
    cmd()
        .args(["route", "review MCP tool", "--limit", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("review").or(predicate::str::contains("MCP")));
}

#[test]
fn test_show_command() {
    cmd()
        .args(["show-command", "review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("review"));
}

#[test]
fn test_show_tool() {
    cmd()
        .args(["show-tool", "MCPTool"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCPTool").or(predicate::str::contains("mcptool")));
}

#[test]
fn test_exec_command() {
    cmd()
        .args(["exec-command", "review", "inspect security review"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mirrored command 'review'"));
}

#[test]
fn test_exec_tool() {
    cmd()
        .args(["exec-tool", "MCPTool", "fetch resource list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mirrored tool 'MCPTool'"));
}

#[test]
fn test_bootstrap_runs() {
    cmd()
        .args(["bootstrap", "review MCP tool", "--limit", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Runtime Session"))
        .stdout(predicate::str::contains("Startup Steps"))
        .stdout(predicate::str::contains("Routed Matches"));
}

#[test]
fn test_turn_loop_runs() {
    cmd()
        .args([
            "turn-loop",
            "review MCP tool",
            "--max-turns",
            "2",
            "--structured-output",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Turn 1"))
        .stdout(predicate::str::contains("stop_reason="));
}

#[test]
fn test_flush_transcript_and_load_session() {
    let output = cmd()
        .args(["flush-transcript", "review MCP tool"])
        .output()
        .expect("run flush-transcript");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("flushed=true"));

    let session_path = stdout.lines().next().unwrap().trim();
    let session_id = std::path::Path::new(session_path)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    cmd()
        .args(["load-session", &session_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&session_id))
        .stdout(predicate::str::contains("messages"));

    let _ = std::fs::remove_file(session_path);
}

#[test]
fn test_remote_mode() {
    cmd()
        .args(["remote-mode", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=remote"));
}

#[test]
fn test_ssh_mode() {
    cmd()
        .args(["ssh-mode", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=ssh"));
}

#[test]
fn test_teleport_mode() {
    cmd()
        .args(["teleport-mode", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=teleport"));
}

#[test]
fn test_direct_connect_mode() {
    cmd()
        .args(["direct-connect-mode", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=direct-connect"));
}

#[test]
fn test_deep_link_mode() {
    cmd()
        .args(["deep-link-mode", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode=deep-link"));
}

#[test]
fn test_command_and_tool_snapshots_are_nontrivial() {
    use rusty_claw::commands::PORTED_COMMANDS;
    use rusty_claw::tools::PORTED_TOOLS;

    assert!(PORTED_COMMANDS.len() >= 150);
    assert!(PORTED_TOOLS.len() >= 100);
}

#[test]
fn test_execution_registry() {
    use rusty_claw::execution_registry::build_execution_registry;

    let registry = build_execution_registry();
    assert!(registry.commands.len() >= 150);
    assert!(registry.tools.len() >= 100);
    assert!(registry
        .command("review")
        .unwrap()
        .execute("review security")
        .contains("Mirrored command"));
    assert!(registry
        .tool("MCPTool")
        .unwrap()
        .execute("fetch mcp resources")
        .contains("Mirrored tool"));
}

#[test]
fn test_subsystem_metadata() {
    use rusty_claw::subsystems::{subsystem_module_count, subsystem_sample_files};

    assert!(subsystem_module_count("utils") > 100);
    assert!(subsystem_module_count("bridge") > 0);
    assert!(subsystem_module_count("assistant") > 0);
    assert!(!subsystem_sample_files("utils").is_empty());
}

#[test]
fn test_permission_context() {
    use rusty_claw::permissions::ToolPermissionContext;

    let ctx = ToolPermissionContext::new(
        &["BashTool".into()],
        &["mcp".into()],
    );
    assert!(ctx.blocks("BashTool"));
    assert!(ctx.blocks("bashtool"));
    assert!(ctx.blocks("MCPTool"));
    assert!(ctx.blocks("mcptool"));
    assert!(!ctx.blocks("FileReadTool"));
}

#[test]
fn test_bootstrap_session_tracks_turn_state() {
    use rusty_claw::runtime::PortRuntime;

    let session = PortRuntime.bootstrap_session("review MCP tool", 5);
    assert!(!session.turn_result.matched_tools.is_empty());
    assert!(session.turn_result.output.contains("Prompt:"));
    assert!(session.turn_result.usage.input_tokens >= 1);
}
