use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub session_id: String,
    pub messages: Vec<String>,
    pub input_tokens: usize,
    pub output_tokens: usize,
}

const DEFAULT_DIR: &str = ".port_sessions";

pub fn save_session(session: &StoredSession, directory: Option<&Path>) -> PathBuf {
    let dir = directory
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DIR));
    fs::create_dir_all(&dir).expect("create session dir");
    let path = dir.join(format!("{}.json", session.session_id));
    let json = serde_json::to_string_pretty(session).expect("serialize session");
    fs::write(&path, json).expect("write session");
    path
}

pub fn load_session(session_id: &str, directory: Option<&Path>) -> StoredSession {
    let dir = directory
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DIR));
    let path = dir.join(format!("{}.json", session_id));
    let data = fs::read_to_string(&path).expect("read session file");
    serde_json::from_str(&data).expect("parse session")
}
