use std::collections::HashMap;
use crate::gui::widgets::input_settings::InputTab;

use crate::gui::widgets::mapping_settings::OutputTab;

#[derive(Default)]
pub struct TabState {
    pub mapping_tabs: HashMap<String, OutputTab>,
    pub input_tabs: HashMap<usize, InputTab>
}

