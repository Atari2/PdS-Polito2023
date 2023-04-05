use std::{io::{BufRead, Seek, Write}, time::UNIX_EPOCH};

use fcntl::{lock_file, unlock_file, FcntlLockType};
use rand::Rng;

use clap::Parser;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SensorData {
    seq: u32,
    values: [f32; 10],
    timestamp: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SensorFileMetadata {
    pub read_head: u64,
    pub write_head: u64,
    buffer_size: u64,
    current_size: u64,
}

pub trait BinPack {
    fn to_bytes<T: Write>(&self, writer: &mut std::io::BufWriter<T>) -> Result<(), std::io::Error>
    where
        Self: Sized;
    fn from_bytes<T: BufRead + Seek>(reader: &mut T) -> Option<Self>
    where
        Self: Sized;
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

impl BinPack for SensorFileMetadata {
    fn from_bytes<T: BufRead + Seek>(reader: &mut T) -> Option<Self> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        match reader.rewind() {
            Ok(_) => (),
            Err(_) => return None,
        }
        match reader.read_exact(&mut buf) {
            Ok(_) => unsafe { Some(std::mem::transmute(buf)) },
            Err(_) => None,
        }
    }
    fn to_bytes<T: Write>(
        &self,
        writer: &mut std::io::BufWriter<T>,
    ) -> Result<(), std::io::Error> {
        let buf: [u8; std::mem::size_of::<Self>()] = unsafe { std::mem::transmute(*self) };
        writer.write_all(&buf)?;
        Ok(())
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

impl BinPack for SensorData {
    fn to_bytes<T: Write>(
        &self,
        writer: &mut std::io::BufWriter<T>,
    ) -> Result<(), std::io::Error> {
        let buf: [u8; std::mem::size_of::<Self>()] = unsafe { std::mem::transmute(*self) };
        writer.write_all(&buf)?;
        Ok(())
    }
    fn from_bytes<T: BufRead + Seek>(
        reader: &mut T,
    ) -> Option<Self> {
        let mut buf = [0u8; std::mem::size_of::<Self>()];
        match reader.read_exact(&mut buf) {
            Ok(_) => unsafe { Some(std::mem::transmute(buf)) },
            Err(_) => None,
        }
    }
}

pub fn sensor_lock_file(file: &std::fs::File) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match lock_file(file, None, Some(FcntlLockType::Write)) {
            Ok(true) => return Ok(()),
            Ok(false) => continue,
            Err(e) => return Err(Box::new(e)),
        }
    }
}

pub fn sensor_unlock_file(file: &std::fs::File) -> Result<(), Box<dyn std::error::Error>> {
    match unlock_file(file, None) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not unlock file",
        ))),
        Err(e) => Err(Box::new(e)),
    }
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
