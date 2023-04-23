use clap::Parser;
use sensors::{
    sensor_lock_file, sensor_unlock_file, simulate_sensor, SensorData, SensorFileMetadata, Args,
};
use std::{
    fs::OpenOptions,
    io::{Seek, Write},
};
use binary_io::BinPack;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&args.file)?;
    const SENSOR_DATA_SIZE: u64 = std::mem::size_of::<SensorData>() as u64;
    const METADATA_SIZE: u64 = std::mem::size_of::<SensorFileMetadata>() as u64;
    let mut sensor_num = 0u32;
    loop {
        sensor_lock_file(&file)?;
        let mut writer = std::io::BufWriter::new(&file);
        let mut reader = std::io::BufReader::new(&file);
        // write data to file
        // always read metadata from start of file.
        reader.rewind()?;
        let mut metadata = match SensorFileMetadata::from_bytes(&mut reader) {
            Some(m) => m,
            None => SensorFileMetadata::from_size(args.samples),
        };
        println!("Read metadata {:?}", metadata);
        writer.seek(std::io::SeekFrom::Start(
            metadata.write_head * SENSOR_DATA_SIZE + METADATA_SIZE,
        ))?;
        let d = simulate_sensor(sensor_num);
        if args.verbose {
            println!("Writing data {:?}", d);
        }
        d.to_bytes(&mut writer)?;
        match metadata.advance_write_head() {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        writer.rewind()?;
        metadata.to_bytes(&mut writer)?;
        writer.flush()?;
        sensor_unlock_file(&file)?;
        sensor_num = (sensor_num + 1) % args.sensors;
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
