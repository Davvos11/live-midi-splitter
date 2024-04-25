use serde::{Deserialize, Serialize};
use crate::backend::common_settings::CommonSettings;
use crate::backend::output_settings::{CcMap, ChannelMap, default_channel_map, default_cc_map, default_filter, VelocityCurve};

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
            velocity_curve: VelocityCurve::default()
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
}

impl Default for InputSettings {
    fn default() -> Self {
        Self::new("".to_string())
    }
}