use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub struct Input {
    pub port_name: String,
    pub connection: MidiInputConnection<()>,
}

impl Input {
    pub fn new<F>(port_name: String, callback: F) -> Self
        where F: FnMut(u64, &[u8]) + Send + 'static
    {
        let input = new_input();
        let connection = Self::connect(input, &port_name, callback);

        Self { port_name, connection }
    }

    fn connect<F>(input: MidiInput, port_name: &String, mut callback: F) -> MidiInputConnection<()>
        where F: FnMut(u64, &[u8]) + Send + 'static
    {
        // Find port by name
        let port = input.ports().iter()
            .find(|p| input.port_name(p).unwrap_or_default() == *port_name)
            .unwrap_or_else(|| panic!("Could not find port {port_name}")).clone();
        // Create connection
        input.connect(
            &port,
            "input",
            move |ts, data, _| callback(ts, data),
            (),
        ).unwrap_or_else(|_| panic!("Could not connect to port {port_name}"))
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
            output.connect(&port, "output")
                .map_err(|_| ConnectError {})
        } else {
            Err(ConnectError {})
        }
    }
}

#[derive(Debug)]
pub struct ConnectError {}

pub fn new_input() -> MidiInput {
    MidiInput::new("Live Midi Splitter input").unwrap()
}

pub fn new_output() -> MidiOutput {
    MidiOutput::new("Live Midi Splitter output").unwrap()
}