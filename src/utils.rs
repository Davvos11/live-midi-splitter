use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use pro_serde_versioned::{VersionedDeserialize, VersionedSerialize, VersionedUpgrade};
use rfd::FileDialog;
use serde::Deserialize;
use crate::backend::properties::{Properties, PropertiesV0_3_0, PropertiesVersioned};

pub fn save_dialog(properties: Arc<Mutex<Properties>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(mut location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .save_file()
    {
        if location.extension().is_none() {
            location.set_extension("lmsc");
        }
        let Ok(file) = File::create(&location) else {
            return None;
        };
        let properties = properties.lock().unwrap();
        let versioned: PropertiesVersioned = properties.to_owned().into();
        let serialised: serde_json::Value = versioned.versioned_serialize().unwrap();
        if serde_json::to_writer_pretty(file, &serialised).is_ok() {
            return Some(location);
        }
    }
    None
}


pub fn load_dialog(properties: Arc<Mutex<Properties>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .pick_file()
    {
        if load(&location, properties) { return Some(location) }
    }
    None
}

pub fn load(location: &PathBuf, properties: Arc<Mutex<Properties>>) -> bool {
    let Ok(file) = File::open(location) else {
      return false;
    };
    let reader = BufReader::new(file);
    if let Ok(data) = serde_json::from_reader(reader) {
        *properties.lock().unwrap() = match PropertiesVersioned::versioned_deserialize::<serde_json::Value>(&data) {
            Ok(versioned_data) => {
                versioned_data.upgrade_to_latest()
            }
            Err(_) => {
                // Try parsing un-versioned file (i.e. before v0.4.0)
                if let Ok(properties_old) = PropertiesV0_3_0::deserialize(data) {
                    PropertiesVersioned::V1(properties_old).upgrade_to_latest()
                } else {
                    return false;
                }
            }
        };

        // TODO move to different tab and refresh view
        return true
    }
    false
}
