use std::f64::consts::E;
use serde::{Deserialize, Serialize};

pub fn default_filter() -> (u8, u8) { (0, 128) }

pub fn default_cc_map() -> CcMap {
    // Entry -1 corresponds to "any other cc", and 0 for "any other channel"
    vec![(0, -1, CcMapping::default())]
}

pub fn default_channel_map() -> ChannelMap {
    // Entry 0 for "any other channel"
    vec![(0, ChannelMapping::default())]
}

pub trait CommonSettings {
    fn key_filter_enabled_mut(&mut self) -> &mut bool;
    fn key_filter_mut(&mut self) -> &mut (u8, u8);
    fn cc_map_mut(&mut self) -> &mut CcMap;
    fn channel_map_mut(&mut self) -> &mut ChannelMap;
    fn velocity_curve_mut(&mut self) -> &mut VelocityCurve;
    fn velocity_range_mut(&mut self) -> &mut VelocityRange;
    fn transpose_mut(&mut self) -> &mut Transpose;

    fn key_filter_enabled(&self) -> bool;
    fn key_filter(&self) -> (u8, u8);
    fn cc_map(&self) -> &CcMap;
    fn channel_map(&self) -> &ChannelMap;
    fn velocity_curve(&self) -> &VelocityCurve;
    fn velocity_range(&self) -> &VelocityRange;
    fn transpose(&self) -> &Transpose;


    fn get_velocity(&self, vel_in: f64) -> f64 {
        let mut vel_in = vel_in;
        let mut floor = 0.0;
        let mut ceil = 127.0;
        let range = self.velocity_range();
        if vel_in < range.min as f64 {
            match range.below_min {
                OutsideRange::Ignore => { return 0.0; }
                OutsideRange::Clamp => { vel_in = range.min as f64 }
                OutsideRange::Scale => {}
            }
        }
        if vel_in > range.max as f64 {
            match range.above_max {
                OutsideRange::Ignore => { return 0.0; }
                OutsideRange::Clamp => { vel_in = range.max as f64 }
                OutsideRange::Scale => {}
            }
        }
        if range.below_min == OutsideRange::Scale {
            floor = range.min as f64;
        }
        if range.above_max == OutsideRange::Scale {
            ceil = range.max as f64;
        }
        // Linearly scale according to floor and ceil
        vel_in = floor + ((ceil - floor) * (vel_in - 1.0) / 126.0);

        match self.velocity_curve() {
            VelocityCurve::Linear => { vel_in }
            VelocityCurve::Fixed(value) => { *value as f64 }
            VelocityCurve::Exponential(exp) => {
                127.0 * (vel_in / 127.0).powf(*exp)
            }
            VelocityCurve::Logarithmic(alpha) => {
                (127.0 / (1.0 + *alpha * 127.0).log2()) * (1.0 + *alpha * vel_in).log2()
            }
            VelocityCurve::SCurve(alpha) => {
                127.0 / (1.0 + E.powf(-alpha / 10.0 * (vel_in - 63.5)))
            }
        }
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
            below_min: OutsideRange::Scale,
            above_max: OutsideRange::Scale,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum OutsideRange {
    Ignore,
    Clamp,
    Scale,
}


#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Transpose {
    pub value: i8,
    pub ignore_global: bool
}
