use std::collections::HashMap;
use crate::gui::widgets::mapping_settings;

pub struct TabState {
    pub mapping_tabs: HashMap<String, mapping_settings::Tab>
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            mapping_tabs: HashMap::new()
        }
    }
}

