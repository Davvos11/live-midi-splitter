use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use crate::backend::output_settings::OutputSettings;

#[derive(Serialize, Deserialize, Clone)]
pub struct Preset {
    pub id: usize,
    pub name: String,
    pub mapping: HashMap<usize, Vec<OutputSettings>>, // [list of outputs for each input]
}

impl Preset {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            mapping: HashMap::new(),
        }
    }

    pub fn new_from_id(id: usize) -> Self {
        Self::new(id, format!("Preset {}", id + 1))
    }
}

impl Hash for Preset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state)
    }
}
