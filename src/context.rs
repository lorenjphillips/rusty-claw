use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct PortContext {
    pub source_root: PathBuf,
    pub tests_root: PathBuf,
    pub assets_root: PathBuf,
    pub archive_root: PathBuf,
    pub python_file_count: usize,
    pub test_file_count: usize,
    pub asset_file_count: usize,
    pub archive_available: bool,
}

fn count_files(dir: &Path, ext: Option<&str>) -> usize {
    if !dir.exists() {
        return 0;
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| match ext {
            Some(x) => e.path().extension().map_or(false, |ex| ex == x),
            None => true,
        })
        .count()
}

pub fn build_port_context(base: Option<&Path>) -> PortContext {
    let root = base
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
    let source_root = root.join("src");
    let tests_root = root.join("tests");
    let assets_root = root.join("assets");
    let archive_root = root.join("archive").join("claude_code_ts_snapshot").join("src");

    PortContext {
        python_file_count: count_files(&source_root, Some("rs")),
        test_file_count: count_files(&tests_root, Some("rs")),
        asset_file_count: count_files(&assets_root, None),
        archive_available: archive_root.exists(),
        source_root,
        tests_root,
        assets_root,
        archive_root,
    }
}

pub fn render_context(ctx: &PortContext) -> String {
    [
        format!("Source root: {}", ctx.source_root.display()),
        format!("Test root: {}", ctx.tests_root.display()),
        format!("Assets root: {}", ctx.assets_root.display()),
        format!("Archive root: {}", ctx.archive_root.display()),
        format!("Source files: {}", ctx.python_file_count),
        format!("Test files: {}", ctx.test_file_count),
        format!("Assets: {}", ctx.asset_file_count),
        format!("Archive available: {}", ctx.archive_available),
    ]
    .join("\n")
}
