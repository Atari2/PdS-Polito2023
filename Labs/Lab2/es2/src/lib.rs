use std::time::UNIX_EPOCH;

use fs2::FileExt;
use rand::Rng;

use clap::Parser;
use binary_io::{BinPack, BinaryIO, Write};

#[derive(Debug)]
pub enum SensorDataError {
    LockError(Box<dyn std::error::Error>),
    IoError(std::io::Error),
    UnlockError(Box<dyn std::error::Error>),
    MetadataReadError,
    DataReadError
}

impl From<std::io::Error> for SensorDataError {
    fn from(e: std::io::Error) -> Self {
        SensorDataError::IoError(e)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, BinaryIO)]
pub struct SensorData {
    seq: u32,
    values: [f32; 10],
    timestamp: u32,
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

#[repr(C)]
#[derive(Debug, Copy, Clone, BinaryIO)]
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

    pub fn advance_write_head(&mut self) -> Result<(), std::io::Error> {
        if self.current_size == self.buffer_size {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Buffer is full",
            ))
        } else {
            self.write_head = (self.write_head + 1) % self.buffer_size;
            self.current_size += 1;
            Ok(())
        }
    }
    pub fn advance_read_head(&mut self) -> Result<(), std::io::Error> {
        if self.current_size == 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Buffer is empty",
            ))
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

pub fn sensor_lock_file(file: &std::fs::File) -> Result<(), SensorDataError> {
    file.lock_exclusive().map_err(|e| SensorDataError::LockError(e.into()))
}

pub fn sensor_unlock_file(file: &std::fs::File) -> Result<(), SensorDataError> {
    file.unlock().map_err(|e| SensorDataError::UnlockError(e.into()))
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
        seq: sensor_num,
        values,
        timestamp,
    }
}

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "sensor_data.bin")]
    pub file: String,
    #[clap(long, default_value = "10")]
    pub sensors: u32,
    #[clap(long, default_value = "100")]
    pub samples: u64,
    #[clap(short, long, default_value = "false")]
    pub verbose: bool,
}
