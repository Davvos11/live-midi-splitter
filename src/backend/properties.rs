use serde::{Deserialize, Serialize};
use crate::backend::input_settings::InputSettings;
use crate::backend::preset::Preset;

#[derive(Serialize, Deserialize)]
pub struct Properties {
    #[serde(skip_serializing, default)]
    pub available_inputs: Vec<String>,
    #[serde(skip_serializing, default)]
    pub available_outputs: Vec<String>,
    pub inputs: Vec<InputSettings>,
    pub presets: Vec<Preset>,
    pub current_preset: usize,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            available_inputs: Vec::new(),
            available_outputs: Vec::new(),
            inputs: vec![InputSettings::default()],
            presets: vec![Preset::default()],
            current_preset: 0,
        }
    }
}