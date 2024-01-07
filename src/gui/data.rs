use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const NAME: Option<&str> = option_env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize, Default)]
pub struct RecentFiles {
    pub files: HashSet<PathBuf>
}

impl RecentFiles {
    pub fn load() -> Option<Self> {
        if let Some(mut location) = dirs::config_dir() {
            location.push(NAME.unwrap_or("live-midi-splitter"));
            location.push("recent_files.json");
            if let Ok(file) = File::open(location) {
                let reader = BufReader::new(file);
                if let Ok(recent_files) = serde_json::from_reader(reader) {
                    return Some(recent_files);
                }
            }
        }
        None
    }

    pub fn save(&self) {
        if let Some(mut location) = dirs::config_dir() {
            location.push(NAME.unwrap_or("live-midi-splitter"));
            // Create path if not exist
            if fs::create_dir_all(&location).is_ok() {
                location.push("recent_files.json");
                if let Ok(file) = File::create(location) {
                    let _ = serde_json::to_writer_pretty(file, self);
                }
            }
        }
    }
}
