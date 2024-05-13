use std::collections::HashMap;
use crate::backend::properties::MidiLearn;
use crate::gui::widgets::input_settings::InputTab;

use crate::gui::widgets::mapping_settings::OutputTab;

#[derive(Default, Clone, Debug)]
pub struct State {
    pub available_inputs: Vec<String>,
    pub available_outputs: Vec<String>,
    pub midi_learn: MidiLearn,
}

#[derive(Default)]
pub struct TabState {
    pub mapping_tabs: HashMap<String, OutputTab>,
    pub input_tabs: HashMap<usize, InputTab>
}

