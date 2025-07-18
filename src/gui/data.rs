use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

const NAME: Option<&str> = option_env!("CARGO_PKG_NAME");

#[derive(Serialize, Deserialize, Default)]
pub struct RecentFiles {
    files: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn add(&mut self, item: PathBuf) {
        self.remove(&item);
        self.files.push(item);
        self.save();
    }

    pub fn remove(&mut self, item: &PathBuf) {
        if let Some(i) = self.files.iter().position(|p| p == item) {
            self.files.remove(i);
            self.save()
        }
    }

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

    fn save(&self) {
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
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }
}
