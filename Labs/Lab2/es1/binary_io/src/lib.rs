pub use binary_io_derive::BinaryIO;

pub use std::io::{BufRead, Write, BufReader, BufWriter, Seek};

pub trait BinPack {
    fn to_bytes<T: std::io::Write>(&self, writer: &mut std::io::BufWriter<T>) -> Result<(), std::io::Error>
    where
        Self: Sized;
    fn from_bytes<T: std::io::BufRead + std::io::Seek>(reader: &mut T) -> Option<Self>
    where
        Self: Sized;
}