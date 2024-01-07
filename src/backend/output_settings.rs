use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

// Serde does not accept default = true, so we make it more stupid to make it work
fn get_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Eq)]
pub struct OutputSettings {
    pub port_name: String,
    #[serde(default = "get_true")]
    pub buffer_pedals: bool,
}

impl OutputSettings {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            buffer_pedals: true,
        }
    }
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

impl Hash for OutputSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.port_name, state)
    }
}

impl PartialEq<Self> for OutputSettings {
    fn eq(&self, other: &Self) -> bool {
        self.port_name == other.port_name
    }
}