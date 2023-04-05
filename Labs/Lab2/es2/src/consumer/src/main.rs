use clap::Parser;
use sensors::{sensor_lock_file, sensor_unlock_file, SensorData, SensorFileMetadata, Args};
use std::{
    fs::OpenOptions,
    io::{Seek, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file;
    loop {
        match OpenOptions::new()
            .write(true)
            .read(true)
            .open(&args.file)
        {
            Ok(f) => {
                file = f;
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                std::thread::sleep(std::time::Duration::from_millis(50)); // wait 50 ms and try opening the file again
            }
        };
    }
    const SENSOR_DATA_SIZE: u64 = std::mem::size_of::<SensorData>() as u64;
    const METADATA_SIZE: u64 = std::mem::size_of::<SensorFileMetadata>() as u64;
    loop {
        sensor_lock_file(&file)?;
        // read data from file
        let mut reader = std::io::BufReader::new(&file);
        let mut writer = std::io::BufWriter::new(&file);
        let mut metadata = match SensorFileMetadata::from_bytes(&mut reader) {
            Some(m) => m,
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not read metadata",
            ))?,
        };
        println!("Read metadata {:?}", metadata);
        let mut data = vec![];
        for _ in 0..10 {
            let offset = metadata.read_head * SENSOR_DATA_SIZE + METADATA_SIZE;
            reader.seek(std::io::SeekFrom::Start(offset))?;
            let d = SensorData::from_bytes(&mut reader)?;
            if args.verbose {
                println!("Read data {:?}", d);
            }
            data.push(d);
            match metadata.advance_read_head() {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
            if metadata.is_empty() {
                break;
            }
        }
        println!("After reading: {:?}", metadata);
        metadata.to_bytes(&mut writer)?;
        reader.rewind()?;
        writer.flush()?;
        sensor_unlock_file(&file)?;
        std::thread::sleep(std::time::Duration::from_millis(10_000));
    }
}
