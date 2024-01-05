use std::collections::HashSet;

#[derive(Clone)]
pub struct Preset {
    pub name: String,
    pub inputs: HashSet<String>,
    pub outputs: HashSet<String>,
}

impl Preset {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inputs: HashSet::new(),
            outputs: HashSet::new(),
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::new("Default preset".to_string())
    }
}
