use serde::{Deserialize, Serialize};

use crate::backend::common_settings::{
    default_cc_map, default_channel_map, default_filter, CcMap, ChannelMap, CommonSettings,
    Transpose, VelocityCurve, VelocityRange,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputSettings {
    pub port_name: String,
    pub use_program_change: bool,
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
    pub velocity_range: VelocityRange,
    #[serde(default)]
    pub transpose: Transpose,
}

impl InputSettings {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            use_program_change: false,
            key_filter_enabled: false,
            key_filter: default_filter(),
            cc_map: default_cc_map(),
            channel_map: default_channel_map(),
            velocity_curve: VelocityCurve::default(),
            velocity_range: VelocityRange::default(),
            transpose: Transpose::default(),
        }
    }
}

impl CommonSettings for InputSettings {
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

    fn transpose_mut(&mut self) -> &mut Transpose {
        &mut self.transpose
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

    fn transpose(&self) -> &Transpose {
        &self.transpose
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self::new("".to_string())
    }
}
