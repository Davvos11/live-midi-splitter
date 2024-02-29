use std::fmt::{Display, Formatter, Pointer, write};
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

// Serde does not accept default = true, so we make it more stupid to make it work
fn get_true() -> bool {
    true
}

fn default_filter() -> (u8, u8) { (0, 128) }

fn default_cc_map() -> CcMap {
    // Entry -1 corresponds to "any other cc", and 0 for "any other channel"
    vec![(0, -1, CcMapping::default())]
}

#[derive(Serialize, Deserialize, Clone, Eq, Debug)]
pub struct OutputSettings {
    pub port_name: String,
    #[serde(default = "get_true")]
    pub buffer_pedals: bool,
    #[serde(default)]
    pub key_filter_enabled: bool,
    #[serde(default = "default_filter")]
    pub key_filter: (u8, u8),
    #[serde(default = "default_cc_map")]
    pub cc_map: CcMap,
}

impl OutputSettings {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            buffer_pedals: true,
            key_filter_enabled: false,
            key_filter: default_filter(),
            cc_map: default_cc_map(),
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

type CcMap = Vec<(u8, i8, CcMapping)>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub enum CcMapping {
    #[default]
    PassThrough,
    PassThroughToChannel(u8),
    MapToCc(u8),
    MapToChannelCc(u8, u8),
    Ignore,
}

impl CcMapping {
    pub fn all() -> &'static [CcMapping; 5] {
        &[CcMapping::PassThrough, CcMapping::Ignore, CcMapping::MapToCc(0), CcMapping::PassThroughToChannel(1), CcMapping::MapToChannelCc(1, 0)]
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            CcMapping::PassThrough => { "Send unmodified" }
            CcMapping::PassThroughToChannel(_) => { "Send to channel" }
            CcMapping::MapToCc(_) => { "Send CC" }
            CcMapping::MapToChannelCc(_, _) => { "Send CC" }
            CcMapping::Ignore => { "Discard" }
        }
    }

    pub fn get_description_with_blanks(&self) -> &'static str {
        match self {
            CcMapping::PassThrough => { "Send unmodified" }
            CcMapping::PassThroughToChannel(_) => { "Send to channel _" }
            CcMapping::MapToCc(_) => { "Send CC _" }
            CcMapping::MapToChannelCc(_, _) => { "Send CC _ to channel _" }
            CcMapping::Ignore => { "Discard" }
        }
    }
}