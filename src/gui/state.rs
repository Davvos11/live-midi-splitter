use crate::backend::pipewire_utils::{pipewire_installed, Pipewire};
use crate::backend::properties::MidiLearn;
use crate::backend::MidiPort;
use crate::gui::widgets::input_settings::InputTab;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::gui::widgets::mapping_settings::OutputTab;

#[derive(Default, Clone, Debug)]
pub struct State {
    pub available_inputs: Vec<MidiPort>,
    pub available_outputs: Vec<MidiPort>,
    pub pipewire_status: Option<Pipewire>,
    pub pipewire_error: Option<String>,
    pub midi_learn: MidiLearn,
    file_path: Option<PathBuf>,
    pub path_changed: bool,
}

impl State {
    pub fn new() -> Self {
        let mut pipewire_status = None;
        let mut pipewire_error = None;
        if pipewire_installed() {
            match Pipewire::new() {
                Ok(pw) => {
                    pipewire_status = Some(pw);
                }
                Err(err) => {
                    pipewire_error = Some(err.to_string());
                }
            }
        }
        
        Self {
            pipewire_status,
            pipewire_error,
            ..Default::default()
        }
    }

    pub fn set_file_path(&mut self, file_path: PathBuf) {
        self.file_path = Some(file_path);
        self.path_changed = true;
    }

    pub fn file_path(&self) -> &Option<PathBuf> {
        &self.file_path
    }

}

#[derive(Default)]
pub struct TabState {
    pub mapping_tabs: HashMap<String, OutputTab>,
    pub input_tabs: HashMap<usize, InputTab>
}

