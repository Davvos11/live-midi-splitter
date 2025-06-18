use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::io;
use std::process::{Command, ExitStatus};
use std::str::FromStr;
use std::time::Instant;

pub fn pipewire_installed() -> bool {
    let pipewire_installed = Command::new("pipewire").arg("--version").status();
    match pipewire_installed {
        Ok(status) => {
            if !status.success() {
                return false;
            }
        }
        Err(_) => {
            return false;
        }
    }

    let pw_metadata_installed = Command::new("pw-metadata").arg("--version").status();
    match pw_metadata_installed {
        Ok(status) => {
            if !status.success() {
                return false;
            }
        }
        Err(_) => {
            return false;
        }
    }

    true
}

pub enum PipewireError {
    IO(io::Error),
    Status(ExitStatus, String),
    ParseError(String),
}

impl From<io::Error> for PipewireError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl Display for PipewireError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PipewireError::IO(err) => err.fmt(f),
            PipewireError::Status(code, msg) => {
                write!(f, "Error {} when executing pw-metadata, {}", code, msg)
            }
            PipewireError::ParseError(value) => {
                write!(f, "Error when parsing pw-metadata output: {value}")
            }
        }
    }
}

fn execute(command: &str, args: &[&str]) -> Result<String, PipewireError> {
    let result = Command::new(command).args(args).output()?;
    let stdout = String::from_utf8_lossy(&result.stdout);

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(PipewireError::Status(
            result.status,
            format!("{stdout}\n{stderr}"),
        ));
    }

    Ok(stdout.to_string())
}

fn set<T: ToString>(setting: &str, value: T) -> Result<(), PipewireError> {
    execute(
        "pw-metadata",
        &["-n", "settings", "0", setting, value.to_string().as_str()],
    )?;
    Ok(())
}

fn get<T: FromStr>(setting: &str) -> Result<T, PipewireError> {
    let result = execute("pw-metadata", &["-n", "settings", "0", setting])?;
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"value:'(\d+)'").unwrap());
    if let Some(groups) = RE.captures(&result) {
        if let Some(value) = groups.get(1) {
            if let Ok(value) = value.as_str().parse() {
                return Ok(value);
            }
        }
    }

    Err(PipewireError::ParseError(result))
}

#[derive(Clone, Debug)]
pub struct Pipewire {
    data: PipeWireData,
    set_buffer_size: Option<u32>,
    last_update: Instant,
}

#[derive(Clone, Copy, Debug)]
pub struct PipeWireData {
    pub buffer_size: u32,
    pub force_buffer_size: Option<u32>,
    pub sample_rate: u32,
    pub delay: f32,
}

impl PipeWireData {
    pub fn new(buffer_size: u32, force_buffer_size: u32, sample_rate: u32) -> Self {
        let force_buffer_size = if force_buffer_size == 0 {None} else {Some(force_buffer_size)};
        let buffer_size = force_buffer_size.unwrap_or(buffer_size);
        Self {
            buffer_size,
            force_buffer_size,
            sample_rate,
            delay: buffer_size as f32 / sample_rate as f32 * 1000.0,
        }
    }
}

impl Pipewire {
    pub fn new() -> Result<Self, PipewireError> {
        let buffer_size = Self::_get_buffer_size()?;
        let force_buffer_size = Self::_get_force_buffer_size()?;
        let sample_rate = Self::_get_sample_rate()?;
        Ok(Self {
            data: PipeWireData::new(buffer_size, force_buffer_size, sample_rate),
            set_buffer_size: None,
            last_update: Instant::now(),
        })
    }

    pub fn should_update(&self) -> Result<bool, PipewireError> {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_secs() > 1 {
            return Ok(true);
        }
        if let Some(new_buffer_size) = self.set_buffer_size {
            Self::_set_buffer_size(new_buffer_size)?;
            return Ok(true);
        }
        Ok(false)
    }
    
    pub fn get_new_data() -> Result<PipeWireData, PipewireError> {
        let buffer_size = Self::_get_buffer_size()?;
        let force_buffer_size = Self::_get_force_buffer_size()?;
        let sample_rate = Self::_get_sample_rate()?;
        Ok(PipeWireData::new(buffer_size, force_buffer_size, sample_rate))
    }
    
    pub fn update(&mut self) -> Result<bool, PipewireError> {
        let update = self.should_update()?;
        if update {
            self.data = Self::get_new_data()?;
            self.last_update = Instant::now();
        }
        Ok(update)
    }
    
    pub fn update_manual(&mut self, data: PipeWireData) -> Result<(), PipewireError> {
        self.data = data;
        self.last_update = Instant::now();
        Ok(())
    }

    pub fn get_values(&mut self) -> PipeWireData {
        self.data
    }

    pub fn set_buffer_size(&mut self, buffer_size: u32) {
        self.set_buffer_size = Some(buffer_size);
    }

    fn _get_buffer_size() -> Result<u32, PipewireError> {
        get("clock.quantum")
    }
    
    fn _get_force_buffer_size() -> Result<u32, PipewireError> {
        get("clock.force-quantum")
    }

    fn _set_buffer_size(size: u32) -> Result<(), PipewireError> {
        set("clock.force-quantum", size)
    }

    fn _reset_buffer_size() -> Result<(), PipewireError> {
        set("clock.force-quantum", 0)
    }

    fn _get_sample_rate() -> Result<u32, PipewireError> {
        get("clock.rate")
    }
}
