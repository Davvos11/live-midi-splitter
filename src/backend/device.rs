use std::fmt::{Debug, Formatter};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use crate::backend::MidiPort;

pub struct Input {
    pub port_name: MidiPort,
    pub connection: MidiInputConnection<usize>,
}

impl Input {
    pub fn new<F>(port_name: MidiPort, callback: F) -> Result<Self, ConnectError>
        where F: FnMut(u64, &[u8], &mut usize) + Send + 'static
    {
        let input = new_input();
        let connection = Self::connect(input, &port_name.internal, callback)?;

        Ok(Self { port_name, connection })
    }

    fn connect<F>(input: MidiInput, port_name: &String, callback: F) -> Result<MidiInputConnection<usize>, ConnectError>
        where F: FnMut(u64, &[u8], &mut usize) + Send + 'static
    {
        // Find port by name
        if let Some(port) = input.ports().iter()
            .find(|p| input.port_name(p).unwrap_or_default() == *port_name) {
            // Create connection
            input.connect(
                port,
                "input",
                callback,
                0,
            ).or(Err(ConnectError {}))
        } else {
            Err(ConnectError {})
        }
    }
}

impl Debug for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<input {}>", self.port_name.readable)
    }
}

pub struct Output {
    pub connection: MidiOutputConnection,
}

impl Output {
    pub fn new(port_name: &String) -> Result<Self, ConnectError> {
        let output = new_output();
        let connection = Self::connect(output, port_name)?;

        Ok(Self { connection })
    }

    fn connect(output: MidiOutput, port_name: &String) -> Result<MidiOutputConnection, ConnectError> {
        // Find port by name
        let ports = output.ports();
        let port = ports.iter()
            .find(|p| output.port_name(p).unwrap_or_default() == *port_name);
        if let Some(port) = port {
            // Create connection
            output.connect(port, "output")
                .map_err(|_| ConnectError {})
        } else {
            Err(ConnectError {})
        }
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<output>")
    }
}

#[derive(Debug)]
pub struct ConnectError {}

pub fn new_input() -> MidiInput {
    MidiInput::new("input Live Midi Splitter").unwrap()
}

pub fn new_output() -> MidiOutput {
    MidiOutput::new("Live Midi Splitter output").unwrap()
}