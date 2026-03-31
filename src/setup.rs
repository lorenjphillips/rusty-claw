use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PrefetchResult {
    pub name: String,
    pub started: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct DeferredInitResult {
    pub trusted: bool,
    pub plugin_init: bool,
    pub skill_init: bool,
    pub mcp_prefetch: bool,
    pub session_hooks: bool,
}

impl DeferredInitResult {
    pub fn as_lines(&self) -> Vec<String> {
        vec![
            format!("- plugin_init={}", self.plugin_init),
            format!("- skill_init={}", self.skill_init),
            format!("- mcp_prefetch={}", self.mcp_prefetch),
            format!("- session_hooks={}", self.session_hooks),
        ]
    }
}

pub fn run_deferred_init(trusted: bool) -> DeferredInitResult {
    DeferredInitResult {
        trusted,
        plugin_init: trusted,
        skill_init: trusted,
        mcp_prefetch: trusted,
        session_hooks: trusted,
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceSetup {
    pub rust_version: String,
    pub implementation: String,
    pub platform_name: String,
    pub test_command: String,
}

impl WorkspaceSetup {
    pub fn startup_steps(&self) -> Vec<&str> {
        vec![
            "start top-level prefetch side effects",
            "build workspace context",
            "load mirrored command snapshot",
            "load mirrored tool snapshot",
            "prepare parity audit hooks",
            "apply trust-gated deferred init",
        ]
    }
}

pub fn build_workspace_setup() -> WorkspaceSetup {
    let version = env!("CARGO_PKG_VERSION");
    WorkspaceSetup {
        rust_version: version.into(),
        implementation: "rustc".into(),
        platform_name: std::env::consts::OS.into(),
        test_command: "cargo test".into(),
    }
}

fn start_mdm_raw_read() -> PrefetchResult {
    PrefetchResult {
        name: "mdm_raw_read".into(),
        started: true,
        detail: "Simulated MDM raw-read prefetch for workspace bootstrap".into(),
    }
}

fn start_keychain_prefetch() -> PrefetchResult {
    PrefetchResult {
        name: "keychain_prefetch".into(),
        started: true,
        detail: "Simulated keychain prefetch for trusted startup path".into(),
    }
}

fn start_project_scan(root: &Path) -> PrefetchResult {
    PrefetchResult {
        name: "project_scan".into(),
        started: true,
        detail: format!("Scanned project root {}", root.display()),
    }
}

#[derive(Debug, Clone)]
pub struct SetupReport {
    pub setup: WorkspaceSetup,
    pub prefetches: Vec<PrefetchResult>,
    pub deferred_init: DeferredInitResult,
    pub trusted: bool,
    pub cwd: PathBuf,
}

impl SetupReport {
    pub fn as_markdown(&self) -> String {
        let mut lines = vec![
            "# Setup Report".into(),
            String::new(),
            format!(
                "- Rust: {} ({})",
                self.setup.rust_version, self.setup.implementation
            ),
            format!("- Platform: {}", self.setup.platform_name),
            format!("- Trusted mode: {}", self.trusted),
            format!("- CWD: {}", self.cwd.display()),
            String::new(),
            "Prefetches:".into(),
        ];
        for p in &self.prefetches {
            lines.push(format!("- {}: {}", p.name, p.detail));
        }
        lines.push(String::new());
        lines.push("Deferred init:".into());
        lines.extend(self.deferred_init.as_lines());
        lines.join("\n")
    }
}

pub fn run_setup(cwd: Option<&Path>, trusted: bool) -> SetupReport {
    let root = cwd
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let prefetches = vec![
        start_mdm_raw_read(),
        start_keychain_prefetch(),
        start_project_scan(&root),
    ];
    SetupReport {
        setup: build_workspace_setup(),
        prefetches,
        deferred_init: run_deferred_init(trusted),
        trusted,
        cwd: root,
    }
}
