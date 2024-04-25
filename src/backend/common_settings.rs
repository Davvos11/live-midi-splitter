use crate::backend::output_settings::{CcMap, ChannelMap, VelocityCurve};

pub trait CommonSettings {
    fn key_filter_enabled_mut(&mut self) -> &mut bool;
    fn key_filter_mut(&mut self) -> &mut (u8, u8);
    fn cc_map_mut(&mut self) -> &mut CcMap;
    fn channel_map_mut(&mut self) -> &mut ChannelMap;
    fn velocity_curve_mut(&mut self) -> &mut VelocityCurve;
    
    fn key_filter_enabled(&self) -> bool;
    fn key_filter(&self) -> (u8, u8);
    fn cc_map(&self) -> &CcMap;
    fn channel_map(&self) -> &ChannelMap;
    fn velocity_curve(&self) -> &VelocityCurve;

}