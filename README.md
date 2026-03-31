<p align="center">
  <img src="assets/rusty-claw-hero.png" alt="rusty-claw" width="500" />
</p>

<p align="center">
  <strong>Claude Code's agent harness architecture, mapped and compiled in Rust</strong>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust&style=flat-square" alt="Rust" /></a>
  <a href="https://github.com/lorenjphillips/rusty-claw/actions"><img src="https://img.shields.io/badge/tests-31%20passing-brightgreen?style=flat-square" alt="Tests" /></a>
  <a href="https://github.com/lorenjphillips/rusty-claw/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License" /></a>
  <a href="https://github.com/lorenjphillips/claw-code"><img src="https://img.shields.io/badge/port%20of-claw--code-blueviolet?style=flat-square" alt="Port of claw-code" /></a>
</p>

---

# rusty-claw

Rust rewrite of [claw-code](https://github.com/lorenjphillips/claw-code) — a clean-room structural map of Claude Code's agent harness architecture, compiled to a single static binary.

## What This Is (and Isn't)

This is an **architectural study**, not a working Claude Code replacement. It maps out how Claude Code organizes its 207 commands, 184 tools, 29 subsystems, prompt routing, session management, and startup sequence — but it does not call any LLM API, execute real tools, or run actual agent loops.

When you run a command like `exec-tool MCPTool "fetch resources"`, it prints what the real system *would* do — it doesn't actually do it. Every execution path is a structural shim that describes the routing and dispatch behavior without performing it.

The original [claw-code](https://github.com/lorenjphillips/claw-code) Python project worked the same way. This is a faithful Rust port of that same metadata mirror.

**What it does:**
- Loads and indexes all 207 commands and 184 tools from mirrored JSON snapshots
- Routes prompts to matching commands/tools via token scoring
- Simulates turn-based conversation loops with budget tracking
- Persists and reloads session state
- Maps the full bootstrap sequence, subsystem graph, and permission model

**What it does not do:**
- Call any LLM API
- Execute shell commands, read files, or perform real tool actions
- Replace or replicate Claude Code as a functional CLI

## Why Rust

The Python version proved the architecture. This version compiles it into a single binary — all 207 commands, 184 tools, and 29 subsystem metadata files baked in at compile time via `include_str!`. No interpreter, no JSON files on disk, no `pip install`.

## Quickstart

```bash
cargo build --release
cargo run -- summary
cargo run -- commands --limit 10
cargo run -- tools --limit 10 --query MCP
cargo run -- route "review MCP tool"
cargo test
```

## Repository Layout

```text
.
├── src/
│   ├── main.rs               # clap CLI — 22 subcommands
│   ├── lib.rs                 # module declarations
│   ├── models.rs              # shared types
│   ├── commands.rs            # 207 mirrored command entries
│   ├── tools.rs               # 184 mirrored tool entries
│   ├── runtime.rs             # prompt routing + session bootstrap
│   ├── query_engine.rs        # turn-based engine with budgets
│   ├── session_store.rs       # JSON session persistence
│   └── ...                    # 13 more modules
├── reference_data/            # JSON snapshots embedded at compile time
│   ├── commands_snapshot.json
│   ├── tools_snapshot.json
│   └── subsystems/*.json
└── tests/
    └── integration.rs         # 31 integration tests
```

## Subcommands

All 22 subcommands from the Python version, same output:

```bash
cargo run -- summary                          # workspace summary
cargo run -- manifest                         # file/module manifest
cargo run -- parity-audit                     # compare against archived TS surface
cargo run -- setup-report                     # startup/prefetch report
cargo run -- bootstrap "prompt"               # full session bootstrap
cargo run -- turn-loop "prompt" --max-turns 3 # simulated multi-turn loop
cargo run -- command-graph                    # builtin/plugin/skill segmentation
cargo run -- tool-pool                        # assembled tool pool
cargo run -- bootstrap-graph                  # 7-stage execution plan
cargo run -- subsystems --limit 16            # subsystem metadata
cargo run -- flush-transcript "prompt"        # persist session transcript
cargo run -- load-session <id>                # reload persisted session
cargo run -- remote-mode <target>             # runtime mode simulation
cargo run -- ssh-mode <target>
```

## Ownership / Affiliation Disclaimer

This repository does **not** claim ownership of the original Claude Code source material. This repository is **not affiliated with, endorsed by, or maintained by Anthropic**.
