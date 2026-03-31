use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptStore {
    pub entries: Vec<String>,
    pub flushed: bool,
}

impl TranscriptStore {
    pub fn append(&mut self, entry: String) {
        self.entries.push(entry);
        self.flushed = false;
    }

    pub fn compact(&mut self, keep_last: usize) {
        if self.entries.len() > keep_last {
            let start = self.entries.len() - keep_last;
            self.entries = self.entries[start..].to_vec();
        }
    }

    pub fn replay(&self) -> Vec<String> {
        self.entries.clone()
    }

    pub fn flush(&mut self) {
        self.flushed = true;
    }
}
