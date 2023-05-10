use std::{time::UNIX_EPOCH, fmt::Display};

use rand::Rng;

use clap::Parser;

#[derive(Debug)]
pub enum SensorDataError {
    BufferFullError,
    BufferEmptyError,
    MetadataReadError,
    DataReadError
}

impl Display for SensorDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorDataError::BufferFullError => write!(f, "Buffer is full"),
            SensorDataError::BufferEmptyError => write!(f, "Buffer is empty"),
            SensorDataError::MetadataReadError => write!(f, "Error reading metadata"),
            SensorDataError::DataReadError => write!(f, "Error reading data"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SensorData {
    _seq: u32,
    values: [f32; 10],
    _timestamp: u32,
}

impl SensorData {
    pub fn min(&self) -> f32 {
        self.values.iter().copied().reduce(f32::min).unwrap_or(0.0f32)
    }
    pub fn max(&self) -> f32 {
        self.values.iter().copied().reduce(f32::max).unwrap_or(f32::MAX)
    }
    pub fn avg(&self) -> f32 {
        let total = match self.values.iter().copied().reduce(|acc, e| acc + e) {
            Some(x) => x,
            None => return 0.0f32
        };
        total / self.values.len() as f32
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SensorFileMetadata {
    pub read_head: u64,
    pub write_head: u64,
    buffer_size: u64,
    current_size: u64,
}

impl Default for SensorFileMetadata {
    fn default() -> Self {
        SensorFileMetadata {
            read_head: 0,
            write_head: 0,
            buffer_size: 20,
            current_size: 0,
        }
    }
}

impl SensorFileMetadata {
    pub fn from_size(size: u64) -> Self {
        SensorFileMetadata {
            read_head: 0,
            write_head: 0,
            buffer_size: size,
            current_size: 0,
        }
    }

    pub fn advance_write_head(&mut self) -> Result<(), SensorDataError> {
        if self.current_size == self.buffer_size {
            Err(SensorDataError::BufferFullError)
        } else {
            self.write_head = (self.write_head + 1) % self.buffer_size;
            self.current_size += 1;
            Ok(())
        }
    }
    pub fn advance_read_head(&mut self) -> Result<(), SensorDataError> {
        if self.current_size == 0 {
            Err(SensorDataError::BufferEmptyError)
        } else {
            self.read_head = (self.read_head + 1) % self.buffer_size;
            self.current_size -= 1;
            Ok(())
        }
    }
    pub fn is_empty(&self) -> bool {
        self.current_size == 0
    }
}

pub struct SensorsBuffer {
    pub metadata: SensorFileMetadata,
    pub buffer: Vec<SensorData>,
}

pub fn simulate_sensor(sensor_num: u32) -> SensorData {
    let mut rng = rand::thread_rng();
    let mut values = [0.0; 10];
    for v in &mut values {
        *v = rng.gen::<f32>();
    }
    let timestamp = match std::time::SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs() as u32,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    SensorData {
        _seq: sensor_num,
        values,
        _timestamp: timestamp,
    }
}

#[derive(Clone, Parser)]
pub struct Args {
    #[clap(long, default_value = "10")]
    pub sensors: u32,
    #[clap(long, default_value = "100")]
    pub samples: u64,
    #[clap(short, long, default_value = "false")]
    pub verbose: bool,
    #[clap(short, long, default_value = "false")]
    pub nowait: bool
}