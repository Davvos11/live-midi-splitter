use pro_serde_versioned::{Upgrade, VersionedDeserialize, VersionedSerialize, VersionedUpgrade};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

use crate::backend::input_settings::InputSettings;
use crate::backend::preset::Preset;

#[derive(Default, Clone, Debug)]
pub struct MidiLearn {
    pub target: Option<usize>,
    pub result: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Properties {
    pub inputs: Vec<InputSettings>,
    pub presets: Vec<Preset>,
    pub current_preset: usize,
    #[serde(default)]
    pub transpose: i8,
    #[serde(default)]
    pub shortcuts: Vec<String>,
}

impl Properties {
    pub fn remove_preset(&mut self, id: usize) {
        self.presets.remove(id);
        // Update "internal" ids to match position in list
        self.presets.iter_mut().enumerate().for_each(|(i, p)| p.id = i);
        self.current_preset = if id > 0 { id - 1 } else { 0 };
    }

    pub fn duplicate_preset(&mut self, id: usize) {
        let new_preset = self.presets.get(id).cloned();
        if let Some(mut preset) = new_preset {
            let re = Regex::new(r"\((\d+)\)$").unwrap();
            if re.find(&preset.name).is_some() {
                preset.name = re.replace(&preset.name, |captures: &Captures| {
                    let num: u32 = captures[1].parse().unwrap();
                    format!("({})", num + 1)
                }).to_string();
            } else {
                preset.name += " (2)";
            }
            self.presets.insert(id + 1, preset);
            // Update "internal" ids to match position in list
            self.presets.iter_mut().enumerate().for_each(|(i, p)| p.id = i);
            self.current_preset = id + 1;
        }
    }
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            inputs: vec![InputSettings::default()],
            presets: vec![Preset::new_from_id(0)],
            current_preset: 0,
            transpose: 0,
            shortcuts: vec![],
        }
    }
}

#[derive(VersionedSerialize, VersionedDeserialize, VersionedUpgrade, Clone, Debug)]
pub enum PropertiesVersioned {
    V1(PropertiesV0_3_0),
    V2(Properties),
}

//////////////////////////////////////////////////////////////////////
//                      Older versions of properties                //
//////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "Properties")]
pub struct PropertiesV0_3_0 {
    pub inputs: Vec<InputSettings>,
    pub presets: Vec<Preset>,
    pub current_preset: usize,
}

impl Upgrade<Properties> for PropertiesV0_3_0 {
    fn upgrade(self) -> Properties {
        Properties {
            inputs: self.inputs,
            presets: self.presets,
            current_preset: self.current_preset,
            ..Properties::default()
        }
    }
}