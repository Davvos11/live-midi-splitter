use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use midir::{MidiInput, MidiInputConnection, MidiIO, MidiOutput, MidiOutputConnection};
use crate::midi::preset::Preset;

pub mod preset;

#[derive(Default)]
pub struct Properties {
    pub available_inputs: Vec<String>,
    pub available_outputs: Vec<String>,
    pub presets: Vec<Preset>,
    pub inputs: Vec<String>,
}

pub struct Backend {
    properties: Arc<Mutex<Properties>>,

    input_listeners: HashMap<String, MidiInputConnection<()>>,
    output_handlers: Arc<Mutex<HashMap<String, MidiOutputConnection>>>
}

impl Backend {
    pub fn new() -> Self {
        Self {
            properties: Arc::new(Mutex::new(Properties::default())),

            input_listeners: HashMap::new(),
            output_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn run(&mut self) {
        // TODO error to frontend (new_input uses unwrap)
        let midi_in = new_input();
        let midi_out = new_output();
        loop {
            let mut properties = self.properties.lock().unwrap();
            // Send available ports to frontend
            properties.available_inputs = get_ports(&midi_in);
            properties.available_outputs = get_ports(&midi_out);

            // Update presets
            for preset in properties.presets.iter() {
                // Remove listeners for removed inputs
                self.input_listeners.retain(|name, _| preset.inputs.contains(name));

                // Get inputs that need to be added
                let new_inputs = preset.inputs.iter()
                    .filter(|&name| !self.input_listeners.contains_key(name));
                let mut new_listeners = HashMap::new();

                for name in new_inputs {
                    // TODO optimise, don't get ports every time
                    let ports = midi_in.ports();
                    let mut port = ports.iter()
                        .filter(|p| &midi_in.port_name(p).unwrap_or_default() == name);
                    if let Some(port) = port.next() {
                        let output_handlers = Arc::clone(&self.output_handlers);
                        let midi_con = new_input().connect(port, "input", move |_, data, _| {
                            for (_, output) in output_handlers.lock().unwrap().iter_mut() {
                                // TODO error handling
                                output.send(data).unwrap();
                            }
                        }, ()).unwrap();
                        new_listeners.insert(name.clone(), midi_con);
                    }
                }

                // Save listeners
                for listener in new_listeners {
                    self.input_listeners.insert(listener.0, listener.1);
                }

                // Remove handlers for removed outputs
                self.output_handlers.lock().unwrap().retain(|name, _| preset.outputs.contains(name));

                // Get outputs that need to be added
                let new_outputs = preset.outputs.iter()
                    .filter(|&name| !self.output_handlers.lock().unwrap().contains_key(name));
                let mut new_handlers = HashMap::new();

                for name in new_outputs {
                    // TODO optimise, don't get ports every time
                    let ports = midi_out.ports();
                    let mut port = ports.iter()
                        .filter(|p| &midi_out.port_name(p).unwrap_or_default() == name);
                    if let Some(port) = port.next() {
                        let midi_con = new_output().connect(port, "output").unwrap();
                        new_handlers.insert(name.clone(), midi_con);
                    }
                }

                // Save listeners
                for handler in new_handlers {
                    self.output_handlers.lock().unwrap().insert(handler.0, handler.1);
                }
            }

            drop(properties);

            thread::sleep(Duration::from_millis(100));
        }
    }


    pub fn properties(&self) -> Arc<Mutex<Properties>> {
        Arc::clone(&self.properties)
    }
}

fn get_ports<T: MidiIO>(midi_io: &T) -> Vec<String> {
    midi_io.ports().iter()
        .map(|p| midi_io.port_name(p).unwrap_or("Cannot get port name".to_string()))
        .filter(|p|!p.starts_with("Live Midi Splitter"))
        .collect()
}

fn new_input() -> MidiInput {
    MidiInput::new("Live Midi Splitter input").unwrap()
}

fn new_output() -> MidiOutput {
    MidiOutput::new("Live Midi Splitter output").unwrap()
}