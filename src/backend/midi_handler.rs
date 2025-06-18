use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex};

use crate::backend::device::{ConnectError, Input, Output};
use crate::backend::midi_handler::filter_map::apply_filter_map;
use crate::backend::properties::Properties;
use crate::backend::MidiPort;
use crate::gui::state::State;
use crate::utils::repaint_gui;
use egui::Context;
use midly::num::{u4, u7};
use midly::{live::LiveEvent, MidiMessage};

mod filter_map;

pub type QueueItems = Vec<(String, Vec<u8>)>;

pub struct Listener {
    pub name: MidiPort,
    pub input_id: usize,
    pub properties: Arc<Mutex<Properties>>,
    pub state: Arc<Mutex<State>>,
    pub gui_ctx: Arc<Mutex<Option<Context>>>,
    pub output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    pub event_buffer: Arc<Mutex<HashMap<LiveEvent<'static>, HashSet<EventBufferItem>>>>,
    pub held_pedals: Arc<Mutex<HashMap<(u4, u7), u7>>>, // (channel, controller): value
    pub queue: Arc<Mutex<BinaryHeap<Reverse<u64>>>>,
    pub event_sender: mpsc::Sender<(u64, QueueItems)>,
}

impl Listener {
    pub fn create(self) -> Result<Input, ConnectError> {
        Input::new(
            self.name,
            move |timestamp, data, previous_preset| {
                // Add the timestamp of this event to the queue (do not add queue items yet)
                self.queue.lock().unwrap().push(Reverse(timestamp));

                // Parse midi data
                let event = LiveEvent::parse(data);
                if let Err(error) = event {
                    eprintln!("Midi parse error: {error}");
                    return;
                }
                let event = event.unwrap();
                let mut send_events = Vec::new();

                {
                    let mut properties = self.properties.lock().unwrap();
                    if let Some(input_settings) = properties.inputs.get(self.input_id) {
                        // Handle program change, if enabled
                        if input_settings.use_program_change {
                            if let LiveEvent::Midi { message: MidiMessage::ProgramChange { program }, .. } = event {
                                // Set preset
                                properties.current_preset = program.as_int() as usize;
                                // Redraw frontend
                                repaint_gui(&self.gui_ctx);
                                // Don't send this data to the mappings
                                return;
                            }
                        }
                    } else {
                        eprintln!("Could not get input settings for input {}", self.input_id)
                    }
                }

                // Get preset
                let preset = {
                    let properties = self.properties.lock().unwrap();
                    properties.presets.get(properties.current_preset).cloned()
                };
                if let Some(mapping) = preset.as_ref().and_then(|p| p.mapping.get(&self.input_id).cloned()) {
                    // Check if we changed presets
                    let changed_preset = preset.as_ref().unwrap().id != *previous_preset;
                    if changed_preset {
                        *previous_preset = preset.as_ref().unwrap().id
                    }

                    // Loop through mappings
                    mapping.iter().for_each(|output| {
                        // Clone data so we can modify it separately for each output mapping
                        let mut data = data.to_owned();
                        {
                            let state = self.state.lock().unwrap();
                            let mut output_handlers = self.output_handlers.lock().unwrap();
                            // Check if the output target has disconnected
                            let output_port = state.available_outputs.iter().find(|p| p.readable == output.port_name);
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
                        }

                        // If we just changed presets, send any held pedal events
                        if changed_preset && output.buffer_pedals {
                            self.held_pedals.lock().unwrap().iter().for_each(|(&(channel, controller), &value)| {
                                let pedal_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value } };
                                let mut data = Vec::new();
                                if pedal_event.write(&mut data).is_ok() {
                                    send_events.push((output.port_name.clone(), data.clone()));
                                }
                            });
                        }

                        let mut send = true;
                        let mut ignore_transpose = output.transpose.ignore_global;
                        {
                            let properties = self.properties.lock().unwrap();
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
                        }

                        if send {
                            send_events.push((output.port_name.clone(), data.clone()));
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
                                    let mut event_buffer = self.event_buffer.lock().unwrap();
                                    event_buffer.entry(off_listen_event).or_default()
                                        .insert(EventBufferItem { output_name: output.port_name.clone(), off_event: off_send_event });
                                }
                                MidiMessage::NoteOff { key, .. } => {
                                    // Remove previously saved event (saved on note-on)
                                    let off_event = LiveEvent::Midi { channel, message: MidiMessage::NoteOff { key, vel: 0.into() } };
                                    let mut event_buffer = self.event_buffer.lock().unwrap();
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
                                                    let mut event_buffer = self.event_buffer.lock().unwrap();
                                                    event_buffer.entry(off_listen_event).or_default()
                                                        .insert(EventBufferItem { output_name: output.port_name.clone(), off_event: off_send_event });
                                                    // Mark pedal as held (so it can be sent on preset switch)
                                                    self.held_pedals.lock().unwrap().insert((channel, controller), value);
                                                } else {
                                                    // Remove previously saved event (saved on note-on)
                                                    let off_event = LiveEvent::Midi { channel, message: MidiMessage::Controller { controller, value: 0.into() } };
                                                    let mut event_buffer = self.event_buffer.lock().unwrap();
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
                } else {
                    let properties = self.properties.lock().unwrap();
                    eprintln!("Could not get output mapping for preset {} input {}", properties.current_preset, self.input_id)
                }

                // Send note-off, after-touch and pedal release events to outputs that are no longer active
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
                    let mut event_buffer = self.event_buffer.lock().unwrap();
                    if let Some(outputs) = event_buffer.remove(&off_event) {
                        // Send to outputs that still need note-off events
                        outputs.iter().for_each(|item| {
                            let mut buf = Vec::new();
                            if let Err(e) = item.off_event.write(&mut buf) {
                                eprintln!("{e}");
                            }
                            send_events.push((item.output_name.clone(), buf.clone()));
                        });
                    }
                }
                // Send events to the queue handler thread
                if let Err(e) = self.event_sender.send((timestamp, send_events)) {
                    eprintln!("Error sending events {e:?}");
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

pub struct QueueHandler {
    priority_queue: Arc<Mutex<BinaryHeap<Reverse<u64>>>>,
    output_handlers: Arc<Mutex<HashMap<String, Output>>>,
    event_queue: HashMap<u64, QueueItems>,
}

impl QueueHandler {
    pub fn run(&mut self, rx: mpsc::Receiver<(u64, QueueItems)>) {
        loop {
            // Receive incoming events
            let (timestamp, events) = rx.recv().unwrap();
            // If this is the next event in the priority queue, send it
            if self.should_send_event(timestamp) {
                self.priority_queue.lock().unwrap().pop();
                self.send_events(&events);
            } else {
                // If this is not the next event, cache it
                self.event_queue.insert(timestamp, events);
            }

            // Check if the next event in the priority queue is in the cache
            if let Some(next_timestamp) = self.priority_queue.lock().unwrap().peek() {
                if let Some(events) = self.event_queue.remove(&next_timestamp.0) {
                    self.send_events(&events);
                }
            }
        }
    }

    fn should_send_event(&mut self, timestamp: u64) -> bool {
        if let Some(next_timestamp) = self.priority_queue.lock().unwrap().peek() {
            if next_timestamp.0 == timestamp {
                return true;
            }
        }
        false
    }

    fn send_events(&self, events: &QueueItems) {
        let mut output_handlers = self.output_handlers.lock().unwrap();
        for (port_name, data) in events {
            output_handlers.get_mut(port_name).unwrap()
                .connection.send(data).unwrap_or_else(|_| eprintln!("Failed to send to {port_name}"));
        }
    }

    pub fn new(output_handlers: Arc<Mutex<HashMap<String, Output>>>, priority_queue: Arc<Mutex<BinaryHeap<Reverse<u64>>>>) -> Self {
        Self { priority_queue, output_handlers, event_queue: HashMap::new() }
    }
}
