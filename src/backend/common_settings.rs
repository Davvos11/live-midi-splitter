use std::f64::consts::E;
use crate::backend::output_settings::{CcMap, ChannelMap, OutsideRange, VelocityCurve, VelocityRange};

pub trait CommonSettings {
    fn key_filter_enabled_mut(&mut self) -> &mut bool;
    fn key_filter_mut(&mut self) -> &mut (u8, u8);
    fn cc_map_mut(&mut self) -> &mut CcMap;
    fn channel_map_mut(&mut self) -> &mut ChannelMap;
    fn velocity_curve_mut(&mut self) -> &mut VelocityCurve;
    fn velocity_range_mut(&mut self) -> &mut VelocityRange;

    fn key_filter_enabled(&self) -> bool;
    fn key_filter(&self) -> (u8, u8);
    fn cc_map(&self) -> &CcMap;
    fn channel_map(&self) -> &ChannelMap;
    fn velocity_curve(&self) -> &VelocityCurve;
    fn velocity_range(&self) -> &VelocityRange;


    fn get_velocity(&self, vel_in: f64) -> f64 {
        let mut vel_in = vel_in;
        let mut floor = 0.0;
        let mut ceil = 127.0;
        let range = self.velocity_range();
        if vel_in < range.min as f64 {
            match range.below_min {
                OutsideRange::Ignore => {return 0.0}
                OutsideRange::Clamp => {vel_in = range.min as f64}
                OutsideRange::Scale => {}
            }
        }
        if vel_in > range.max as f64 {
            match range.above_max {
                OutsideRange::Ignore => {return 0.0}
                OutsideRange::Clamp => {vel_in = range.max as f64}
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
        vel_in = floor + ((ceil-floor)*(vel_in-1.0)/126.0);
            
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