use clap::{Parser, Subcommand};
use std::process;

use rusty_claw::bootstrap_graph::build_bootstrap_graph;
use rusty_claw::command_graph::build_command_graph;
use rusty_claw::commands::{execute_command, get_command, get_commands, render_command_index};
use rusty_claw::modes::{run_deep_link, run_direct_connect, run_remote_mode, run_ssh_mode, run_teleport_mode};
use rusty_claw::parity_audit::run_parity_audit;
use rusty_claw::permissions::ToolPermissionContext;
use rusty_claw::port_manifest::build_port_manifest;
use rusty_claw::query_engine::QueryEnginePort;
use rusty_claw::runtime::PortRuntime;
use rusty_claw::session_store::load_session;
use rusty_claw::setup::run_setup;
use rusty_claw::subsystems::render_subsystems;
use rusty_claw::tool_pool::assemble_tool_pool;
use rusty_claw::tools::{execute_tool, get_tool, get_tools, render_tool_index};

#[derive(Parser)]
#[command(about = "Rust porting workspace for the Claude Code rewrite effort")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Summary,
    Manifest,
    #[command(name = "parity-audit")]
    ParityAudit,
    #[command(name = "setup-report")]
    SetupReport,
    #[command(name = "command-graph")]
    CommandGraph,
    #[command(name = "tool-pool")]
    ToolPool,
    #[command(name = "bootstrap-graph")]
    BootstrapGraph,
    Subsystems {
        #[arg(long, default_value_t = 32)]
        limit: usize,
    },
    Commands {
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = false)]
        no_plugin_commands: bool,
        #[arg(long, default_value_t = false)]
        no_skill_commands: bool,
    },
    Tools {
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value_t = false)]
        simple_mode: bool,
        #[arg(long, default_value_t = false)]
        no_mcp: bool,
        #[arg(long)]
        deny_tool: Vec<String>,
        #[arg(long)]
        deny_prefix: Vec<String>,
    },
    Route {
        prompt: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
    Bootstrap {
        prompt: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },
    #[command(name = "turn-loop")]
    TurnLoop {
        prompt: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long, default_value_t = 3)]
        max_turns: usize,
        #[arg(long, default_value_t = false)]
        structured_output: bool,
    },
    #[command(name = "flush-transcript")]
    FlushTranscript {
        prompt: String,
    },
    #[command(name = "load-session")]
    LoadSession {
        session_id: String,
    },
    #[command(name = "remote-mode")]
    RemoteMode {
        target: String,
    },
    #[command(name = "ssh-mode")]
    SshMode {
        target: String,
    },
    #[command(name = "teleport-mode")]
    TeleportMode {
        target: String,
    },
    #[command(name = "direct-connect-mode")]
    DirectConnectMode {
        target: String,
    },
    #[command(name = "deep-link-mode")]
    DeepLinkMode {
        target: String,
    },
    #[command(name = "show-command")]
    ShowCommand {
        name: String,
    },
    #[command(name = "show-tool")]
    ShowTool {
        name: String,
    },
    #[command(name = "exec-command")]
    ExecCommand {
        name: String,
        prompt: String,
    },
    #[command(name = "exec-tool")]
    ExecTool {
        name: String,
        payload: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = run(cli.command);
    process::exit(code);
}

fn run(cmd: Commands) -> i32 {
    match cmd {
        Commands::Summary => {
            let manifest = build_port_manifest(None);
            let engine = QueryEnginePort::new(manifest);
            println!("{}", engine.render_summary());
            0
        }
        Commands::Manifest => {
            let manifest = build_port_manifest(None);
            println!("{}", manifest.to_markdown());
            0
        }
        Commands::ParityAudit => {
            println!("{}", run_parity_audit().to_markdown());
            0
        }
        Commands::SetupReport => {
            println!("{}", run_setup(None, true).as_markdown());
            0
        }
        Commands::CommandGraph => {
            println!("{}", build_command_graph().as_markdown());
            0
        }
        Commands::ToolPool => {
            println!("{}", assemble_tool_pool(false, true, None).as_markdown());
            0
        }
        Commands::BootstrapGraph => {
            println!("{}", build_bootstrap_graph().as_markdown());
            0
        }
        Commands::Subsystems { limit } => {
            println!("{}", render_subsystems(limit));
            0
        }
        Commands::Commands {
            limit,
            query,
            no_plugin_commands,
            no_skill_commands,
        } => {
            if let Some(q) = &query {
                println!("{}", render_command_index(limit, Some(q)));
            } else {
                let commands = get_commands(!no_plugin_commands, !no_skill_commands);
                let mut output_lines = vec![format!("Command entries: {}", commands.len()), String::new()];
                for m in commands.iter().take(limit) {
                    output_lines.push(format!("- {} — {}", m.name, m.source_hint));
                }
                println!("{}", output_lines.join("\n"));
            }
            0
        }
        Commands::Tools {
            limit,
            query,
            simple_mode,
            no_mcp,
            deny_tool,
            deny_prefix,
        } => {
            if let Some(q) = &query {
                println!("{}", render_tool_index(limit, Some(q)));
            } else {
                let permission_context = ToolPermissionContext::new(&deny_tool, &deny_prefix);
                let tools = get_tools(simple_mode, !no_mcp, Some(&permission_context));
                let mut output_lines = vec![format!("Tool entries: {}", tools.len()), String::new()];
                for m in tools.iter().take(limit) {
                    output_lines.push(format!("- {} — {}", m.name, m.source_hint));
                }
                println!("{}", output_lines.join("\n"));
            }
            0
        }
        Commands::Route { prompt, limit } => {
            let matches = PortRuntime.route_prompt(&prompt, limit);
            if matches.is_empty() {
                println!("No mirrored command/tool matches found.");
            } else {
                for m in &matches {
                    println!("{}\t{}\t{}\t{}", m.kind, m.name, m.score, m.source_hint);
                }
            }
            0
        }
        Commands::Bootstrap { prompt, limit } => {
            println!("{}", PortRuntime.bootstrap_session(&prompt, limit).as_markdown());
            0
        }
        Commands::TurnLoop {
            prompt,
            limit,
            max_turns,
            structured_output,
        } => {
            let results = PortRuntime.run_turn_loop(&prompt, limit, max_turns, structured_output);
            for (idx, result) in results.iter().enumerate() {
                println!("## Turn {}", idx + 1);
                println!("{}", result.output);
                println!("stop_reason={}", result.stop_reason);
            }
            0
        }
        Commands::FlushTranscript { prompt } => {
            let mut engine = QueryEnginePort::from_workspace();
            engine.submit_message(&prompt, &[], &[], &[]);
            let path = engine.persist_session();
            println!("{}", path);
            println!("flushed={}", engine.transcript_store.flushed);
            0
        }
        Commands::LoadSession { session_id } => {
            let session = load_session(&session_id, None);
            println!(
                "{}\n{} messages\nin={} out={}",
                session.session_id,
                session.messages.len(),
                session.input_tokens,
                session.output_tokens
            );
            0
        }
        Commands::RemoteMode { target } => {
            println!("{}", run_remote_mode(&target).as_text());
            0
        }
        Commands::SshMode { target } => {
            println!("{}", run_ssh_mode(&target).as_text());
            0
        }
        Commands::TeleportMode { target } => {
            println!("{}", run_teleport_mode(&target).as_text());
            0
        }
        Commands::DirectConnectMode { target } => {
            println!("{}", run_direct_connect(&target).as_text());
            0
        }
        Commands::DeepLinkMode { target } => {
            println!("{}", run_deep_link(&target).as_text());
            0
        }
        Commands::ShowCommand { name } => {
            match get_command(&name) {
                Some(m) => {
                    println!("{}\n{}\n{}", m.name, m.source_hint, m.responsibility);
                    0
                }
                None => {
                    println!("Command not found: {}", name);
                    1
                }
            }
        }
        Commands::ShowTool { name } => {
            match get_tool(&name) {
                Some(m) => {
                    println!("{}\n{}\n{}", m.name, m.source_hint, m.responsibility);
                    0
                }
                None => {
                    println!("Tool not found: {}", name);
                    1
                }
            }
        }
        Commands::ExecCommand { name, prompt } => {
            let result = execute_command(&name, &prompt);
            println!("{}", result.message);
            if result.handled { 0 } else { 1 }
        }
        Commands::ExecTool { name, payload } => {
            let result = execute_tool(&name, &payload);
            println!("{}", result.message);
            if result.handled { 0 } else { 1 }
        }
    }
}
