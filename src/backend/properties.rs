use crate::backend::input_settings::InputSettings;
use crate::backend::preset::Preset;

pub struct Properties {
    pub available_inputs: Vec<String>,
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