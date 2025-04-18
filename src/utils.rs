use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use pro_serde_versioned::{VersionedDeserialize, VersionedSerialize, VersionedUpgrade};
use rfd::FileDialog;
use serde::Deserialize;
use crate::backend::properties::{Properties, PropertiesV0_3_0, PropertiesVersioned};
use crate::gui::tabs::Tab;

pub fn save_dialog(properties: Arc<Mutex<Properties>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(mut location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .save_file()
    {
        if location.extension().is_none() {
            location.set_extension("lmsc");
        }
        if save(&location, properties) {
            return Some(location);
        }
    }
    None
}

pub fn save(location: &PathBuf, properties: Arc<Mutex<Properties>>) -> bool {
    let Ok(file) = File::create(location) else {
        return false;
    };
    let properties = properties.lock().unwrap();
    let versioned: PropertiesVersioned = properties.to_owned().into();
    let serialised: serde_json::Value = versioned.versioned_serialize().unwrap();
    if serde_json::to_writer_pretty(file, &serialised).is_ok() {
        return true;
    }
    false
}

pub fn load_dialog(properties: Arc<Mutex<Properties>>, current_tab: Arc<Mutex<Tab>>) -> Option<PathBuf> {
    // TODO error handling
    if let Some(location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .pick_file()
    {
        if load(&location, properties, current_tab) { return Some(location); }
    }
    None
}

pub fn load(location: &PathBuf, properties: Arc<Mutex<Properties>>, current_tab: Arc<Mutex<Tab>>) -> bool {
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

        *current_tab.lock().unwrap() = Tab::QuickStart;
        // TODO refresh view
        return true;
    }
    false
}

const NOTE_NAMES: &[&str; 12] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const NOTE_NAMES_FLAT: &[&str; 12] = &["C", "DB", "D", "EB", "E", "F", "GB", "G", "AB", "A", "BB", "B"];

pub fn midi_to_note(note: u8) -> String {
    let octave = (note / 12) as i8 - 1;
    let note_index = (note % 12) as usize;
    format!("{}{}", NOTE_NAMES[note_index], octave)
}

pub fn note_to_midi(string: &str) -> Option<f64> {
    if string.len() < 2 {
        return None;
    }
    let (note, octave) = string.split_at(string.len() - 1);
    let note = note.to_uppercase();
    let note_index =
        NOTE_NAMES.iter().position(|&s| s == note)
            .or(NOTE_NAMES_FLAT.iter().position(|&s| s == note))?;
    let octave = match octave.parse::<i8>() {
        Ok(octave) => {if octave < -1  { return None } else { octave }}
        Err(_) => {return None}
    };

    Some(((octave + 1) * 12 + note_index as i8) as f64)
}
