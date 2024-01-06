use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct InputSettings {
    pub port_name: String,
    pub use_program_change: bool,
}

impl InputSettings {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            use_program_change: false,
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self::new("".to_string())
    }
}