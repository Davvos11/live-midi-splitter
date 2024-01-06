use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Preset {
    pub name: String,
    pub mapping: HashMap<usize, Vec<String>> // [list of outputs for each input]
}

impl Preset {
    pub fn new(name: String) -> Self {
        Self {
            name,
            mapping: HashMap::new()
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::new("Default preset".to_string())
    }
}
