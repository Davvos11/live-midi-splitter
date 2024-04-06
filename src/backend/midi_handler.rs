use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use egui::Context;
use midly::{live::LiveEvent, MidiMessage};
use midly::num::{u4, u7};
use crate::backend::device::{ConnectError, Input, Output};
use crate::backend::output_settings::{CcMapping, ChannelMapping, OutputSettings};
use crate::backend::properties::Properties;

pub fn create_new_listener(
    name: String,
    input_id: usize,
    properties: Arc<Mutex<Properties>>,
    gui_ctx: Arc<Mutex<Option<Context>>>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<OutputSettings>>>>,
    held_pedals: Arc<Mutex<HashMap<(u4, u7), u7>>>, // (channel, controller): value
) -> Result<Input, ConnectError> {
    Input::new(
        name,
        move |_, data, previous_preset| {
            let mut properties = properties.lock().unwrap();

            // Parse midi data
            let event = LiveEvent::parse(data);
            if let Err(error) = event {
                eprintln!("Midi parse error: {error}");
                return;
            }
            let event = event.unwrap();

            if let Some(input_settings) = properties.inputs.get(input_id) {
                // Handle program change, if enabled
                if input_settings.use_program_change {
                    if let LiveEvent::Midi { message: MidiMessage::ProgramChange { program }, .. } = event {
                        // Set preset
                        properties.current_preset = program.as_int() as usize;
                        // Redraw frontend
                        let ctx = gui_ctx.lock().unwrap();
                        if let Some(ctx) = ctx.deref() {
                            ctx.request_repaint();
                        }
                        // Don't send this data to the mappings
                        return;
                    }
                }
            } else {
                eprintln!("Could not get input settings for input {input_id}")
            }

            // Make mutable copy of data
            let mut data = data.to_vec();
            // Apply transpose
            if properties.transpose != 0 {
                if let LiveEvent::Midi { message, .. } = event {
                    match message {
                        MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. } | MidiMessage::Aftertouch { .. } => {
                            // Change raw data directly. data[1] is the key value. set to 0 at underflow
                            data[1] = data[1].checked_add_signed(properties.transpose).unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }

            let mut output_handlers = output_handlers.lock().unwrap();
            // Get preset
            let preset = properties.presets.get(properties.current_preset);
            if let Some(mapping) = preset.and_then(|p| p.mapping.get(&input_id)) {
                // Check if we changed presets
                let changed_preset = preset.unwrap().id != *previous_preset;
                if changed_preset {
                    *previous_preset = preset.unwrap().id
                }

                // Loop through mappings
                mapping.iter().for_each(|output| {
                    // Check if the output target has disconnected
                    if !properties.available_outputs.contains(&output.port_name) {
                        output_handlers.remove(&output.port_name);
                        return;
                    }

                    // Find output_handler or create new
                    if !output_handlers.contains_key(&output.port_name) {
                        // Try to connect
                        let new_handler = Output::new(&output.port_name);
                        match new_handler {
                            Ok(handler) => {
                                output_handlers.insert(output.port_name.clone(), handler);
                            }
                            Err(_) => { return; }
                        }
                    }

                    // If we just changed presets, send any held pedal events
                    if changed_preset && output.buffer_pedals {
                        held_pedals.lock().unwrap().iter().for_each(|(&(channel, controller), &value)| {
                            let pedal_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value } };
                            let mut data = Vec::new();
                            if pedal_event.write(&mut data).is_ok() {
                                output_handlers.get_mut(&output.port_name).unwrap()
                                    .connection.send(&data).unwrap_or_else(|_| println!("Failed to send to {}", output.port_name));
                            }
                        });
                    }

                    // Apply filter
                    let mut send = true;
                    if let LiveEvent::Midi { channel, message } = event {
                        match message {
                            MidiMessage::NoteOn { key, .. } | MidiMessage::NoteOff {key, ..} => {
                                if output.key_filter_enabled &&
                                    (output.key_filter.0 > key || output.key_filter.1 < key) {
                                    send = false;
                                }
                                let channel_map = output.channel_map.iter().find(|(ch, _)| *ch == channel.as_int())
                                    .or(output.channel_map.last());
                                if let Some((_, channel_map)) = channel_map {
                                    match channel_map {
                                        ChannelMapping::PassThrough => {}
                                        ChannelMapping::Channel(new_channel) => {
                                            // We use the difference because the channel is set in the last 4 bits of this byte
                                            // The first 4 bits are always 1011 for cc messages
                                            // channel is 0..=15, new_channel is 1..=16
                                            data[0] = data[0] - channel.as_int() + new_channel - 1;
                                        }
                                        ChannelMapping::Ignore => {send = false}
                                    }
                                }
                            }
                            MidiMessage::Controller { controller, .. } => {
                                let map =
                                    output.cc_map.iter().find(|(ch, cc, _)| *ch == channel.as_int() && *cc as u8 == controller.as_int())
                                        .or(output.cc_map.iter().find(|(ch, cc, _)| *ch == 0 && *cc as u8 == controller.as_int()))
                                        .or(output.cc_map.iter().find(|(ch, cc, _)| *ch == channel.as_int() && *cc == -1))
                                        .or(output.cc_map.last());
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
                                        CcMapping::Ignore => {send = false}
                                    }
                                } else {
                                    println!("Error: no mapping found for cc item, but default should always exist as last item")
                                }
                            }
                            _ => {}
                        }
                    }

                    if send {
                        // Send data
                        // We can unwrap because we checked or inserted the item above
                        output_handlers.get_mut(&output.port_name).unwrap()
                            .connection.send(&data).unwrap_or_else(|_| println!("Failed to send to {}", output.port_name));
                    }

                    // If this is a note-on or pedal event, save it
                    // If this is a note-off or pedal release event, remove previously saved event
                    let mut event_buffer = event_buffer.lock().unwrap();
                    if let LiveEvent::Midi { channel, message } = event {
                        match message {
                            MidiMessage::NoteOn { key, .. } => {
                                // Save corresponding note off event to listen for
                                let off_event = LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } };
                                event_buffer.entry(off_event).or_default().insert(output.clone());
                            }
                            MidiMessage::NoteOff { key, .. } => {
                                // Remove previously saved event (saved on note-on)
                                let off_event = LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } };
                                if let Some(outputs) = event_buffer.get_mut(&off_event) {
                                    outputs.remove(output);
                                }
                            }
                            MidiMessage::Controller { controller, value } => {
                                match controller.as_int() {
                                    64 | 66 | 69 => {
                                        if output.buffer_pedals {
                                            if value >= 64 {
                                                // Save corresponding pedal off event to listen for
                                                let off_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } };
                                                event_buffer.entry(off_event).or_default().insert(output.clone());
                                                // Mark pedal as held (so it can be sent on preset switch)
                                                held_pedals.lock().unwrap().insert((channel, controller), value);
                                            } else {
                                                // Remove previously saved event (saved on note-on)
                                                let off_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } };
                                                if let Some(outputs) = event_buffer.get_mut(&off_event) {
                                                    outputs.remove(output);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                });

                // Send note-off, after-touch and pedal release events to outputs that are no longer active
                let mut event_buffer = event_buffer.lock().unwrap();
                let mut off_event = None;
                let mut is_pedal_event = false;
                if let LiveEvent::Midi { channel, message } = event {
                    match message {
                        MidiMessage::NoteOff { key, .. } |
                        MidiMessage::Aftertouch { key, .. } => {
                            off_event = Some(LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } });
                            // NOTE: after-touch events for held notes should also get sent to previous outputs
                            // I have not tested this, since I do not own a keyboard with after-touch
                        }
                        MidiMessage::Controller { controller, value } => {
                            match controller.as_int() {
                                64 | 66 | 69 => {
                                    if value < 64 {
                                        off_event = Some(
                                            LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } }
                                        );
                                        is_pedal_event = true;
                                        // Mark pedal as released
                                        held_pedals.lock().unwrap().remove(&(channel, controller));
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                };
                if let Some(off_event) = off_event {
                    if let Some(outputs) = event_buffer.get(&off_event) {
                        // Send to outputs that still need note-off events
                        outputs.iter().for_each(|output| {
                            if !output.buffer_pedals && is_pedal_event {
                                return;
                            }
                            output_handlers.get_mut(&output.port_name).unwrap()
                                .connection.send(&data).unwrap_or_else(|_| println!("Failed to send to {}", output.port_name));
                        });
                        if outputs.is_empty() { event_buffer.remove(&off_event); }
                    }
                }
            } else {
                eprintln!("Could not get output mapping for preset {} input {input_id}", properties.current_preset)
            }
        },
    )
}