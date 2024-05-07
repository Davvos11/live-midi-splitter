use midly::live::LiveEvent;
use midly::MidiMessage;

use crate::backend::common_settings::{CcMapping, ChannelMapping};
use crate::backend::common_settings::CommonSettings;

pub fn apply_filter_map(data: &mut [u8], send: &mut bool, settings: &impl CommonSettings) {
    {
        // Parse midi data
        let event = LiveEvent::parse(data);
        if let Err(error) = event {
            eprintln!("Midi parse error: {error}");
            return;
        }
        let event = event.unwrap();
        
        if let LiveEvent::Midi { channel, message } = event {
            match message {
                MidiMessage::NoteOn { key, vel } | MidiMessage::NoteOff { key, vel } => {
                    if settings.key_filter_enabled() &&
                        (settings.key_filter().0 > key || settings.key_filter().1 < key) {
                        *send = false;
                    }
                    let channel_map = settings.channel_map().iter().find(|(ch, _)| *ch == channel.as_int() + 1)
                        .or(settings.channel_map().last());
                    if let Some((_, channel_map)) = channel_map {
                        match channel_map {
                            ChannelMapping::PassThrough => {}
                            ChannelMapping::Channel(new_channel) => {
                                // We use the difference because the channel is set in the last 4 bits of this byte
                                // The first 4 bits are always 1011 for cc messages
                                // channel is 0..=15, new_channel is 1..=16
                                data[0] = data[0] - channel.as_int() + new_channel - 1;
                            }
                            ChannelMapping::Ignore => { *send = false }
                        }
                    }
                    if vel != 0 {
                        let x = settings.get_velocity(vel.as_int() as f64);
                        data[2] = f64::max(1.0, x) as u8
                    }
                }
                MidiMessage::Controller { controller, .. } => {
                    let map =
                        settings.cc_map().iter().find(|(ch, cc, _)| *ch == channel.as_int() && *cc as u8 == controller.as_int())
                            .or(settings.cc_map().iter().find(|(ch, cc, _)| *ch == 0 && *cc as u8 == controller.as_int()))
                            .or(settings.cc_map().iter().find(|(ch, cc, _)| *ch == channel.as_int() && *cc == -1))
                            .or(settings.cc_map().last());
                    if let Some((_, _, map)) = map {
                        match map {
                            CcMapping::PassThroughToChannel(new_channel) => {
                                // We use the difference because the channel is set in the last 4 bits of this byte
                                // The first 4 bits are always 1011 for cc messages
                                // channel is 0..=15, new_channel is 1..=16
                                data[0] = data[0] - channel.as_int() + new_channel - 1;
                            }
                            CcMapping::MapToCc(new_cc) => {
                                data[1] = *new_cc;
                            }
                            CcMapping::MapToChannelCc(new_channel, new_cc) => {
                                data[0] = data[0] - channel.as_int() + new_channel - 1;
                                data[1] = *new_cc;
                            }
                            CcMapping::PassThrough => {}
                            CcMapping::Ignore => { *send = false }
                        }
                    } else {
                        println!("Error: no mapping found for cc item, but default should always exist as last item")
                    }
                }
                _ => {}
            }
        }
    }
} 