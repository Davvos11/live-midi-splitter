use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use crate::backend::common_settings::CommonSettings;
use crate::backend::output_settings::OutsideRange::Scale;

// Serde does not accept default = true, so we make it more stupid to make it work
fn get_true() -> bool {
    true
}

pub fn default_filter() -> (u8, u8) { (0, 128) }

pub fn default_cc_map() -> CcMap {
    // Entry -1 corresponds to "any other cc", and 0 for "any other channel"
    vec![(0, -1, CcMapping::default())]
}

pub fn default_channel_map() -> ChannelMap {
    // Entry 0 for "any other channel"
    vec![(0, ChannelMapping::default())]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    #[serde(default = "default_channel_map")]
    pub channel_map: ChannelMap,
    #[serde(default)]
    pub velocity_curve: VelocityCurve,
    #[serde(default)]
    pub velocity_range: VelocityRange
}

impl OutputSettings {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            buffer_pedals: true,
            key_filter_enabled: false,
            key_filter: default_filter(),
            cc_map: default_cc_map(),
            channel_map: default_channel_map(),
            velocity_curve: VelocityCurve::default(),
            velocity_range: VelocityRange::default(),
        }
    }
}

impl CommonSettings for OutputSettings {
    fn key_filter_enabled_mut(&mut self) -> &mut bool {
        &mut self.key_filter_enabled
    }

    fn key_filter_mut(&mut self) -> &mut (u8, u8) {
        &mut self.key_filter
    }

    fn cc_map_mut(&mut self) -> &mut CcMap {
        &mut self.cc_map
    }

    fn channel_map_mut(&mut self) -> &mut ChannelMap {
        &mut self.channel_map
    }

    fn velocity_curve_mut(&mut self) -> &mut VelocityCurve {
        &mut self.velocity_curve
    }

    fn velocity_range_mut(&mut self) -> &mut VelocityRange {
        &mut self.velocity_range
    }

    fn key_filter_enabled(&self) -> bool {
        self.key_filter_enabled
    }

    fn key_filter(&self) -> (u8, u8) {
        self.key_filter
    }

    fn cc_map(&self) -> &CcMap {
        &self.cc_map
    }

    fn channel_map(&self) -> &ChannelMap {
        &self.channel_map
    }

    fn velocity_curve(&self) -> &VelocityCurve {
        &self.velocity_curve
    }

    fn velocity_range(&self) -> &VelocityRange {
        &self.velocity_range
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

pub type CcMap = Vec<(u8, i8, CcMapping)>;

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

pub type ChannelMap = Vec<(u8, ChannelMapping)>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub enum ChannelMapping {
    #[default]
    PassThrough,
    Channel(u8),
    Ignore,
}

impl ChannelMapping {
    pub fn all() -> &'static [ChannelMapping; 3] {
        &[ChannelMapping::PassThrough, ChannelMapping::Channel(1), ChannelMapping::Ignore]
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            ChannelMapping::PassThrough => { "Send unmodified" }
            ChannelMapping::Channel(_) => { "Send to channel" }
            ChannelMapping::Ignore => { "Discard" }
        }
    }

    pub fn get_description_with_blanks(&self) -> &'static str {
        match self {
            ChannelMapping::PassThrough => { "Send unmodified" }
            ChannelMapping::Channel(_) => { "Send to channel" }
            ChannelMapping::Ignore => { "Discard" }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum VelocityCurve {
    #[default]
    Linear,
    Fixed(u8),
    Exponential(f64),
    Logarithmic(f64),
    SCurve(f64),
}



#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct VelocityRange {
    pub min: u8,
    pub max: u8,
    pub below_min: OutsideRange,
    pub above_max: OutsideRange,
}

impl Default for VelocityRange {
    fn default() -> Self {
        Self {
            min: 1,
            max: 127,
            below_min: Scale,
            above_max: Scale,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum OutsideRange {
    Ignore,
    Clamp,
    Scale
}
