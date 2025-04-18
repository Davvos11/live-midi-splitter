use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use egui::Context;
use midly::{live::LiveEvent, MidiMessage};
use midly::num::{u4, u7};

use crate::backend::device::{ConnectError, Input, Output};
use crate::backend::midi_handler::filter_map::apply_filter_map;
use crate::backend::MidiPort;
use crate::backend::properties::Properties;
use crate::gui::state::State;

mod filter_map;

pub struct Listener {
    pub name: MidiPort,
    pub input_id: usize,
    pub properties: Arc<Mutex<Properties>>,
    pub state: Arc<Mutex<State>>,
    pub gui_ctx: Arc<Mutex<Option<Context>>>,
    pub output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    pub event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<EventBufferItem>>>>,
    pub held_pedals: Arc<Mutex<HashMap<(u4, u7), u7>>>, // (channel, controller): value
}

impl Listener {
    pub fn create(self) -> Result<Input, ConnectError> {
        Input::new(
            self.name,
            move |_, data, previous_preset| {
                let mut properties = self.properties.lock().unwrap();
                let state = self.state.lock().unwrap();

                // Parse midi data
                let event = LiveEvent::parse(data);
                if let Err(error) = event {
                    eprintln!("Midi parse error: {error}");
                    return;
                }
                let event = event.unwrap();

                if let Some(input_settings) = properties.inputs.get(self.input_id) {
                    // Handle program change, if enabled
                    if input_settings.use_program_change {
                        if let LiveEvent::Midi { message: MidiMessage::ProgramChange { program }, .. } = event {
                            // Set preset
                            properties.current_preset = program.as_int() as usize;
                            // Redraw frontend
                            let ctx = self.gui_ctx.lock().unwrap();
                            if let Some(ctx) = ctx.deref() {
                                ctx.request_repaint();
                            }
                            // Don't send this data to the mappings
                            return;
                        }
                    }
                } else {
                    eprintln!("Could not get input settings for input {}", self.input_id)
                }

                let mut output_handlers = self.output_handlers.lock().unwrap();
                // Get preset
                let preset = properties.presets.get(properties.current_preset);
                if let Some(mapping) = preset.and_then(|p| p.mapping.get(&self.input_id)) {
                    // Check if we changed presets
                    let changed_preset = preset.unwrap().id != *previous_preset;
                    if changed_preset {
                        *previous_preset = preset.unwrap().id
                    }

                    // Loop through mappings
                    mapping.iter().for_each(|output| {
                        // Clone data so we can modify it separately for each output mapping
                        let mut data = data.to_owned();
                        // Check if the output target has disconnected
                        let output_port =  state.available_outputs.iter().find(|p| p.readable == output.port_name);
                        if output_port.is_none() {
                            output_handlers.remove(&output.port_name);
                            return;
                        }
                        let output_port = output_port.unwrap(); // safe because of if above

                        // Find output_handler or create new
                        if !output_handlers.contains_key(&output.port_name) {
                            // Try to connect
                            let new_handler = Output::new(output_port);
                            match new_handler {
                                Ok(handler) => {
                                    output_handlers.insert(output.port_name.clone(), handler);
                                }
                                Err(_) => { return; }
                            }
                        }

                        // If we just changed presets, send any held pedal events
                        if changed_preset && output.buffer_pedals {
                            self.held_pedals.lock().unwrap().iter().for_each(|(&(channel, controller), &value)| {
                                let pedal_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value } };
                                let mut data = Vec::new();
                                if pedal_event.write(&mut data).is_ok() {
                                    output_handlers.get_mut(&output.port_name).unwrap()
                                        .connection.send(&data).unwrap_or_else(|_| println!("Failed to send to {}", output.port_name));
                                }
                            });
                        }

                        let mut send = true;
                        let mut ignore_transpose = output.transpose.ignore_global;
                        if let Some(input_settings) = properties.inputs.get(self.input_id) {
                            apply_filter_map(&mut data, &mut send, input_settings);
                            ignore_transpose |= input_settings.transpose.ignore_global;
                        }
                        apply_filter_map(&mut data, &mut send, output);

                        // Apply global transpose
                        if properties.transpose != 0 && !ignore_transpose {
                            if let LiveEvent::Midi { message: MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. } | MidiMessage::Aftertouch { .. }, .. } = event {
                                // Change raw data directly. data[1] is the key value. set to 0 at underflow
                                data[1] = data[1].checked_add_signed(properties.transpose).unwrap_or(0);
                            }
                        }

                        if send {
                            // Send data
                            // We can unwrap because we checked or inserted the item above
                            output_handlers.get_mut(&output.port_name).unwrap()
                                .connection.send(&data).unwrap_or_else(|_| println!("Failed to send to {}", output.port_name));
                        }

                        // Parse midi data (again, after filter/mapping)
                        let event_after = LiveEvent::parse(&data);
                        if let Err(error) = event_after {
                            eprintln!("Midi parse error: {error}");
                            return;
                        }
                        let event_after = event_after.unwrap();

                        // If this is a note-on or pedal event, save it
                        // If this is a note-off or pedal release event, remove previously saved event
                        let mut event_buffer = self.event_buffer.lock().unwrap();
                        if let LiveEvent::Midi { channel, message } = event {
                            match message {
                                MidiMessage::NoteOn { key, .. } => {
                                    // Save corresponding note off event to listen for and to send
                                    let off_listen_event = LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } };
                                    let off_send_event = {
                                        if let LiveEvent::Midi { channel, message: MidiMessage::NoteOn { key, .. } } = event_after {
                                            LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } }
                                        } else {
                                            eprintln!("Event before and after don' t match");
                                            return;
                                        }
                                    };
                                    event_buffer.entry(off_listen_event).or_default()
                                        .insert(EventBufferItem { output_name: output.port_name.clone(), off_event: off_send_event });
                                }
                                MidiMessage::NoteOff { key, .. } => {
                                    // Remove previously saved event (saved on note-on)
                                    let off_event = LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } };
                                    if let Some(outputs) = event_buffer.get_mut(&off_event) {
                                        if let Some(item) = outputs.iter().find(|i| i.output_name == output.port_name).cloned() {
                                            // Only remove event if it was meant to go to the same output channel as us
                                            if same_channel(event_after, item.off_event) {
                                                outputs.remove(&item);
                                            }
                                        }
                                    }
                                }
                                MidiMessage::Controller { controller, value } => {
                                    match controller.as_int() {
                                        64 | 66 | 69 => {
                                            if output.buffer_pedals {
                                                if value >= 64 {
                                                    // Save corresponding note off event to listen for and to send
                                                    let off_listen_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } };
                                                    let off_send_event = {
                                                        if let LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, .. } } = event_after {
                                                            LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } }
                                                        } else {
                                                            eprintln!("Event before and after don' t match");
                                                            return;
                                                        }
                                                    };
                                                    event_buffer.entry(off_listen_event).or_default()
                                                        .insert(EventBufferItem { output_name: output.port_name.clone(), off_event: off_send_event });
                                                    // Mark pedal as held (so it can be sent on preset switch)
                                                    self.held_pedals.lock().unwrap().insert((channel, controller), value);
                                                } else {
                                                    // Remove previously saved event (saved on note-on)
                                                    let off_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } };
                                                    if let Some(outputs) = event_buffer.get_mut(&off_event) {
                                                        if let Some(item) = outputs.iter().find(|i| i.output_name == output.port_name).cloned() {
                                                            // Only remove event if it was meant to go to the same output channel as us
                                                            if same_channel(event_after, item.off_event) {
                                                                outputs.remove(&item);
                                                            }
                                                        }
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
                    let mut event_buffer = self.event_buffer.lock().unwrap();
                    let mut off_event = None;
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
                                            // Mark pedal as released
                                            self.held_pedals.lock().unwrap().remove(&(channel, controller));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    };
                    if let Some(off_event) = off_event {
                        // Get outputs that need this off event _and_ remove it from the buffer.
                        if let Some(outputs) = event_buffer.remove(&off_event) {
                            // Send to outputs that still need note-off events
                            outputs.iter().for_each(|item| {
                                let mut buf = Vec::new();
                                if let Err(e) = item.off_event.write(&mut buf) {
                                    eprintln!("{e}");
                                }
                                output_handlers.get_mut(&item.output_name).unwrap()
                                    .connection.send(&buf)
                                    .unwrap_or_else(|_| println!("Failed to send to {}", &item.output_name));
                            });
                        }
                    }
                } else {
                    eprintln!("Could not get output mapping for preset {} input {}", properties.current_preset, self.input_id)
                }
            },
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EventBufferItem {
    output_name: String,
    off_event: LiveEvent<'static>,
}


fn same_channel(event_a: LiveEvent, event_b: LiveEvent) -> bool {
    if let LiveEvent::Midi { channel: channel_b, .. } = event_b {
        if let LiveEvent::Midi { channel: channel_a, .. } = event_a {
            return channel_b == channel_a;
        }
    }
    false
}
