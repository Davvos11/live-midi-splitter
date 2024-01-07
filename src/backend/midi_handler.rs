use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use egui::ahash::HashSet;
use egui::Context;
use midly::{live::LiveEvent, MidiMessage};
use crate::backend::device::{ConnectError, Input, Output};
use crate::backend::properties::Properties;

mod event_buffer;

pub fn create_new_listener(
    name: String,
    input_id: usize,
    properties: Arc<Mutex<Properties>>,
    gui_ctx: Arc<Mutex<Option<Context>>>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<String>>>>
) -> Result<Input, ConnectError> {
    Input::new(
        name,
        move |_, data| {
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

            let mut output_handlers = output_handlers.lock().unwrap();
            // Get preset
            let preset = properties.presets.get(properties.current_preset);
            if let Some(mapping) = preset.and_then(|p| p.mapping.get(&input_id)) {
                // Loop through mappings
                mapping.iter().for_each(|output_name| {
                    // Check if the output target has disconnected
                    if !properties.available_outputs.contains(output_name) {
                        output_handlers.remove(output_name);
                        return;
                    }

                    // Find output_handler or create new
                    if !output_handlers.contains_key(output_name) {
                        // Try to connect
                        let new_handler = Output::new(output_name);
                        match new_handler {
                            Ok(handler) => {
                                output_handlers.insert(output_name.clone(), handler);
                            }
                            Err(_) => { return; }
                        }
                    }
                    // Send data
                    // We can unwrap because we checked or inserted the item above
                    output_handlers.get_mut(output_name).unwrap()
                        .connection.send(data).unwrap_or_else(|_| println!("Failed to send to {}", output_name));

                    // If this is a note-on or sustain-pedal event, save it
                    // If this is a note-off or sustain-pedal release event, remove previously saved event
                    let mut event_buffer = event_buffer.lock().unwrap();
                    if let LiveEvent::Midi { channel, message } = event {
                        match message {
                            MidiMessage::NoteOff { key, .. } => {
                                let off_event = LiveEvent::Midi {channel, message: MidiMessage::NoteOff {key, vel: 0.into()}};
                                if let Some(outputs) = event_buffer.get_mut(&off_event) {
                                    outputs.remove(output_name);
                                }
                            }
                            MidiMessage::NoteOn { key, .. } => {
                                // Save corresponding note off event to listen for
                                let off_event = LiveEvent::Midi {channel, message: MidiMessage::NoteOff {key, vel: 0.into()}};
                                event_buffer.entry(off_event).or_default().insert(output_name.clone());
                            }
                            MidiMessage::Controller { .. } => {}
                            _ => {}
                        }
                    }
                });

                // Send note-off and sustain-pedal release events to outputs that are no longer active
                let mut event_buffer = event_buffer.lock().unwrap();
                if let LiveEvent::Midi { channel, message } = event {
                    match message {
                        MidiMessage::NoteOff { key, .. } => {
                            let off_event = LiveEvent::Midi {channel, message: MidiMessage::NoteOff {key, vel: 0.into()}};
                            if let Some(outputs) = event_buffer.get(&off_event) {
                                // Send to outputs that still need note-off events
                                outputs.iter().for_each(|output_name| {
                                    output_handlers.get_mut(output_name).unwrap()
                                        .connection.send(data).unwrap_or_else(|_| println!("Failed to send to {}", output_name));
                                });
                                if outputs.is_empty() { event_buffer.remove(&off_event); }
                            }
                        }
                        MidiMessage::Controller { .. } => {}
                        _ => {}
                    }
                }
            } else {
                eprintln!("Could not get output mapping for preset {} input {input_id}", properties.current_preset)
            }
        },
    )
}