use serde::Deserialize;
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Deserialize)]
struct SubsystemMeta {
    #[serde(default)]
    module_count: usize,
    #[serde(default)]
    sample_files: Vec<String>,
}

static SUBSYSTEM_DATA: Lazy<HashMap<String, SubsystemMeta>> = Lazy::new(|| {
    let pairs: Vec<(&str, &str)> = vec![
        ("assistant", include_str!("../reference_data/subsystems/assistant.json")),
        ("bootstrap", include_str!("../reference_data/subsystems/bootstrap.json")),
        ("bridge", include_str!("../reference_data/subsystems/bridge.json")),
        ("buddy", include_str!("../reference_data/subsystems/buddy.json")),
        ("cli", include_str!("../reference_data/subsystems/cli.json")),
        ("components", include_str!("../reference_data/subsystems/components.json")),
        ("constants", include_str!("../reference_data/subsystems/constants.json")),
        ("coordinator", include_str!("../reference_data/subsystems/coordinator.json")),
        ("entrypoints", include_str!("../reference_data/subsystems/entrypoints.json")),
        ("hooks", include_str!("../reference_data/subsystems/hooks.json")),
        ("keybindings", include_str!("../reference_data/subsystems/keybindings.json")),
        ("memdir", include_str!("../reference_data/subsystems/memdir.json")),
        ("migrations", include_str!("../reference_data/subsystems/migrations.json")),
        ("moreright", include_str!("../reference_data/subsystems/moreright.json")),
        ("native_ts", include_str!("../reference_data/subsystems/native_ts.json")),
        ("outputStyles", include_str!("../reference_data/subsystems/outputStyles.json")),
        ("plugins", include_str!("../reference_data/subsystems/plugins.json")),
        ("remote", include_str!("../reference_data/subsystems/remote.json")),
        ("schemas", include_str!("../reference_data/subsystems/schemas.json")),
        ("screens", include_str!("../reference_data/subsystems/screens.json")),
        ("server", include_str!("../reference_data/subsystems/server.json")),
        ("services", include_str!("../reference_data/subsystems/services.json")),
        ("skills", include_str!("../reference_data/subsystems/skills.json")),
        ("state", include_str!("../reference_data/subsystems/state.json")),
        ("types", include_str!("../reference_data/subsystems/types.json")),
        ("upstreamproxy", include_str!("../reference_data/subsystems/upstreamproxy.json")),
        ("utils", include_str!("../reference_data/subsystems/utils.json")),
        ("vim", include_str!("../reference_data/subsystems/vim.json")),
        ("voice", include_str!("../reference_data/subsystems/voice.json")),
    ];
    let mut map = HashMap::new();
    for (name, json_str) in pairs {
        if let Ok(meta) = serde_json::from_str::<SubsystemMeta>(json_str) {
            map.insert(name.to_string(), meta);
        }
    }
    map
});

pub fn subsystem_names() -> Vec<String> {
    let mut names: Vec<String> = SUBSYSTEM_DATA.keys().cloned().collect();
    names.sort();
    names
}

pub fn subsystem_module_count(name: &str) -> usize {
    SUBSYSTEM_DATA
        .get(name)
        .map(|m| m.module_count)
        .unwrap_or(0)
}

pub fn subsystem_sample_files(name: &str) -> Vec<String> {
    SUBSYSTEM_DATA
        .get(name)
        .map(|m| m.sample_files.clone())
        .unwrap_or_default()
}

pub fn render_subsystems(limit: usize) -> String {
    let mut lines = Vec::new();
    let names = subsystem_names();
    for name in names.iter().take(limit) {
        let count = subsystem_module_count(name);
        lines.push(format!("{}\t{}\tRust port placeholder package", name, count));
    }
    lines.join("\n")
}
