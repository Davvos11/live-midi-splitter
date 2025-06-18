pub mod input_settings;
pub mod preset;
pub mod quick_start;
pub mod recent_files;

#[derive(PartialEq)]
pub enum Tab {
    RecentFiles,
    InputSettings,
    QuickStart,
    Preset(usize),
}

impl Default for Tab {
    fn default() -> Self {
        Self::RecentFiles
    }
}
