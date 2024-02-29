use std::collections::HashMap;
use crate::gui::widgets::mapping_settings;

#[derive(Default)]
pub struct TabState {
    pub mapping_tabs: HashMap<String, mapping_settings::Tab>
}

