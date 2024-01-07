use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rfd::FileDialog;
use crate::backend::properties::Properties;

pub fn save_dialog(properties: Arc<Mutex<Properties>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(mut location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .save_file()
    {
        if location.extension().is_none() {
            location.set_extension("lmsc");
        }
        let file = File::create(&location).unwrap();
        let properties = properties.lock().unwrap();
        serde_json::to_writer_pretty(file, properties.deref()).unwrap();
        return Some(location);
    }
    None
}


pub fn load_dialog(properties: Arc<Mutex<Properties>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .pick_file()
    {
        let file = File::open(&location).unwrap();
        let reader = BufReader::new(file);
        *properties.lock().unwrap() = serde_json::from_reader(reader).unwrap();
        return Some(location)
    }
    None
}

pub fn load(location: &PathBuf, properties: Arc<Mutex<Properties>>) -> bool {
    let file = File::open(location).unwrap();
    let reader = BufReader::new(file);
    if let Some(data) = serde_json::from_reader(reader).unwrap() {
        *properties.lock().unwrap() = data;
        // TODO move to different tab and refresh view
        return true
    }
    false
}
