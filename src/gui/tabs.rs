pub mod input_settings;
pub mod preset;

#[derive(PartialEq)]
pub enum Tab {
    InputSettings,
    Preset(usize)
}

impl Default for Tab {
    fn default() -> Self {
        Self::InputSettings
    }
}
