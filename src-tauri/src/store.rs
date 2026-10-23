//! JSON-backed persistence for settings and download history.
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub download_dir: String,
    pub max_concurrent: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub file: Option<String>,
    pub dir: String,
    pub size: u64,
    pub finished_at: u64,
}

pub struct Store {
    dir: PathBuf,
    settings: Mutex<Settings>,
    history: Mutex<Vec<HistoryEntry>>,
}

fn read<T: DeserializeOwned>(path: &PathBuf) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

impl Store {
    pub fn load(dir: PathBuf, default_download_dir: String) -> Self {
        let _ = fs::create_dir_all(&dir);
        let settings = read(&dir.join("settings.json")).unwrap_or(Settings {
            download_dir: default_download_dir,
            max_concurrent: 2,
        });
        let history = read(&dir.join("history.json")).unwrap_or_default();
        Store { dir, settings: Mutex::new(settings), history: Mutex::new(history) }
    }

    fn write<T: Serialize>(&self, name: &str, value: &T) {
        if let Ok(bytes) = serde_json::to_vec_pretty(value) {
            let _ = fs::write(self.dir.join(name), bytes);
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn save_settings(&self, s: Settings) {
        self.write("settings.json", &s);
        *self.settings.lock().unwrap() = s;
    }

    pub fn history(&self) -> Vec<HistoryEntry> {
        self.history.lock().unwrap().clone()
    }

    pub fn push_history(&self, e: HistoryEntry) {
        let mut h = self.history.lock().unwrap();
        h.insert(0, e);
        h.truncate(1000);
        self.write("history.json", &*h);
    }

    pub fn clear_history(&self) {
        let mut h = self.history.lock().unwrap();
        h.clear();
        self.write("history.json", &*h);
    }
}
