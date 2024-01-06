use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use egui::Ui;
use rfd::FileDialog;
use crate::backend::properties::Properties;

pub fn save_load(ui: &mut Ui, properties: &Arc<Mutex<Properties>>, loading: &Arc<Mutex<bool>>) {
    ui.horizontal(|ui| {
        if ui.button("Open").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                load(properties);
                *loading.lock().unwrap() = false;
            });
        }

        if ui.button("Save").clicked() {
            let loading = Arc::clone(loading);
            let properties = Arc::clone(properties);
            let _ = thread::spawn(move || {
                *loading.lock().unwrap() = true;
                save(properties);
                *loading.lock().unwrap() = false;
            });
        }
    });
}

fn save(properties: Arc<Mutex<Properties>>) {
    // TODO error handling
    if let Some(location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .save_file()
    {
        let mut location = location;
        if location.extension().is_none() {
            location.set_extension("lmsc");
        }
        let file = File::create(location).unwrap();
        let properties = properties.lock().unwrap();
        serde_json::to_writer_pretty(file, properties.deref()).unwrap();
    }
}


fn load(properties: Arc<Mutex<Properties>>) {
    // TODO error handling
    if let Some(location) = FileDialog::new()
        .add_filter("Live MIDI splitter config", &["lmsc"])
        .pick_file()
    {
        let file = File::open(location).unwrap();
        let reader = BufReader::new(file);
        *properties.lock().unwrap() = serde_json::from_reader(reader).unwrap();
    }
}