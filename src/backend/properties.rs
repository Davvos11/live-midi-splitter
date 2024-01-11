use pro_serde_versioned::{Upgrade, VersionedDeserialize, VersionedSerialize, VersionedUpgrade};
use serde::{Deserialize, Serialize};
use crate::backend::input_settings::InputSettings;
use crate::backend::preset::Preset;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Properties {
    #[serde(skip_serializing, default)]
    pub available_inputs: Vec<String>,
    #[serde(skip_serializing, default)]
    pub available_outputs: Vec<String>,
    pub inputs: Vec<InputSettings>,
    pub presets: Vec<Preset>,
    pub current_preset: usize,
    #[serde(default)]
    pub transpose: i8,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            available_inputs: Vec::new(),
            available_outputs: Vec::new(),
            inputs: vec![InputSettings::default()],
            presets: vec![Preset::new_from_id(0)],
            current_preset: 0,
            transpose: 0,
        }
    }
}

#[derive(VersionedSerialize, VersionedDeserialize, VersionedUpgrade, Clone, Debug)]
pub enum PropertiesVersioned {
    V1(PropertiesV0_3_0),
    V2(Properties)
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