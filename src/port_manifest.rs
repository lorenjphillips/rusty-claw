use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::models::Subsystem;

#[derive(Debug, Clone)]
pub struct PortManifest {
    pub src_root: PathBuf,
    pub total_rust_files: usize,
    pub top_level_modules: Vec<Subsystem>,
}

impl PortManifest {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("Port root: `{}`", self.src_root.display()),
            format!("Total Rust files: **{}**", self.total_rust_files),
            String::new(),
            "Top-level Rust modules:".into(),
        ];
        for m in &self.top_level_modules {
            lines.push(format!(
                "- `{}` ({} files) — {}",
                m.name, m.file_count, m.notes
            ));
        }
        lines.join("\n")
    }
}

pub fn build_port_manifest(src_root: Option<&Path>) -> PortManifest {
    let root = src_root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("src"));

    let files: Vec<PathBuf> = if root.exists() {
        WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().map_or(false, |ext| ext == "rs")
            })
            .map(|e| e.into_path())
            .collect()
    } else {
        Vec::new()
    };

    let mut counter: HashMap<String, usize> = HashMap::new();
    for path in &files {
        if let Ok(rel) = path.strip_prefix(&root) {
            let key = rel
                .components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .unwrap_or_default();
            *counter.entry(key).or_default() += 1;
        }
    }

    let notes_map: HashMap<&str, &str> = [
        ("main.rs", "CLI entrypoint"),
        ("lib.rs", "module declarations"),
        ("models.rs", "shared data types"),
        ("query_engine.rs", "port orchestration summary layer"),
        ("commands.rs", "command backlog metadata"),
        ("tools.rs", "tool backlog metadata"),
        ("port_manifest.rs", "workspace manifest generation"),
    ]
    .iter()
    .cloned()
    .collect();

    let mut modules: Vec<(String, usize)> = counter.into_iter().collect();
    modules.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let top_level_modules: Vec<Subsystem> = modules
        .into_iter()
        .map(|(name, count)| {
            let notes = notes_map
                .get(name.as_str())
                .unwrap_or(&"Rust port support module")
                .to_string();
            Subsystem {
                path: format!("src/{}", name),
                name,
                file_count: count,
                notes,
            }
        })
        .collect();

    PortManifest {
        src_root: root,
        total_rust_files: files.len(),
        top_level_modules,
    }
}
