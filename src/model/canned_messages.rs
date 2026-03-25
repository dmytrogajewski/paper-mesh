//! User-defined quick-reply message presets.
//!
//! Stored in: <data_dir>/canned_messages.json

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CannedMessage {
    pub label: String,
    pub text: String,
}

#[derive(Debug)]
pub(crate) struct CannedMessages {
    messages: Vec<CannedMessage>,
    path: PathBuf,
}

impl CannedMessages {
    pub(crate) fn load(data_dir: &PathBuf) -> Self {
        let path = data_dir.join("canned_messages.json");
        let messages = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Self::defaults()),
                Err(_) => Self::defaults(),
            }
        } else {
            Self::defaults()
        };
        Self { messages, path }
    }

    fn defaults() -> Vec<CannedMessage> {
        vec![
            CannedMessage {
                label: "OK".into(),
                text: "OK".into(),
            },
            CannedMessage {
                label: "On my way".into(),
                text: "On my way".into(),
            },
            CannedMessage {
                label: "Need help".into(),
                text: "Need help at my location".into(),
            },
            CannedMessage {
                label: "All clear".into(),
                text: "All clear here".into(),
            },
            CannedMessage {
                label: "Check in".into(),
                text: "Checking in - all good".into(),
            },
        ]
    }

    pub(crate) fn messages(&self) -> &[CannedMessage] {
        &self.messages
    }

    pub(crate) fn add(&mut self, label: &str, text: &str) {
        self.messages.push(CannedMessage {
            label: label.to_string(),
            text: text.to_string(),
        });
        self.save();
    }

    pub(crate) fn remove(&mut self, index: usize) {
        if index < self.messages.len() {
            self.messages.remove(index);
            self.save();
        }
    }

    pub(crate) fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.messages) {
            if let Err(e) = fs::write(&self.path, data) {
                log::error!("Failed to save canned messages: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_defaults() {
        let tmp = TempDir::new().unwrap();
        let cm = CannedMessages::load(&tmp.path().to_path_buf());
        assert!(!cm.messages().is_empty());
        assert_eq!(cm.messages()[0].label, "OK");
    }

    #[test]
    fn test_add_and_persist() {
        let tmp = TempDir::new().unwrap();
        let mut cm = CannedMessages::load(&tmp.path().to_path_buf());
        let initial = cm.messages().len();
        cm.add("Custom", "My custom message");
        assert_eq!(cm.messages().len(), initial + 1);

        // Reload from disk
        let cm2 = CannedMessages::load(&tmp.path().to_path_buf());
        assert_eq!(cm2.messages().len(), initial + 1);
        assert_eq!(cm2.messages().last().unwrap().text, "My custom message");
    }

    #[test]
    fn test_remove() {
        let tmp = TempDir::new().unwrap();
        let mut cm = CannedMessages::load(&tmp.path().to_path_buf());
        let initial = cm.messages().len();
        cm.remove(0);
        assert_eq!(cm.messages().len(), initial - 1);
    }

    #[test]
    fn test_load_corrupted_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("canned_messages.json");
        fs::write(&path, "not json").unwrap();
        let cm = CannedMessages::load(&tmp.path().to_path_buf());
        // Should fall back to defaults
        assert!(!cm.messages().is_empty());
    }
}
