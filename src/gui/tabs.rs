pub mod input_settings;
pub mod preset;
pub mod recent_files;

#[derive(PartialEq)]
pub enum Tab {
    RecentFiles,
    InputSettings,
    Preset(usize)
}

impl Default for Tab {
    fn default() -> Self {
        Self::RecentFiles
    }
}
