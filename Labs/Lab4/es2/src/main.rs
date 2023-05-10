use clap::Parser;
use es2::{simulate_sensor, Args, SensorData, SensorFileMetadata, SensorsBuffer};
use std::sync::{Arc, Mutex, MutexGuard};

type SyncSensorBuffer = Arc<Mutex<SensorsBuffer>>;

fn wait_unlock_resource<T>(resource: &'_ Arc<Mutex<T>>) -> MutexGuard<'_, T> {
    loop {
        if let Ok(guard) = resource.try_lock() {
            return guard;
        }
    }
}

fn producer(buffer: SyncSensorBuffer, args: Args) {
    let mut sensor_num = 0u32;
    loop {
        {
            // scope to make sure the mutex is dropped and unlocked before sleeping
            let mut locked_buffer = wait_unlock_resource(&buffer);
            println!("Read metadata {:?}", locked_buffer.metadata);
            let d = simulate_sensor(sensor_num);
            if args.verbose {
                println!("Writing data {:?}", d);
            }
            let index = locked_buffer.metadata.write_head as usize;
            locked_buffer.buffer[index] = d;
            match locked_buffer.metadata.advance_write_head() {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            sensor_num = (sensor_num + 1) % args.sensors;
        }
        if !args.nowait {
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}

fn consumer(buffer: SyncSensorBuffer, args: Args) {
    loop {
        {
            // scope to make sure the mutex is dropped and unlocked before sleeping
            let mut locked_buffer = wait_unlock_resource(&buffer);
            println!("Read metadata {:?}", locked_buffer.metadata);
            let mut data = vec![];
            for _ in 0..args.sensors {
                let d = locked_buffer.buffer[locked_buffer.metadata.read_head as usize];
                if args.verbose {
                    println!("Read data {:?}", d);
                }
                data.push(d);
                match locked_buffer.metadata.advance_read_head() {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                }
                if locked_buffer.metadata.is_empty() {
                    break;
                }
            }
            for (i, sensor) in data.iter().enumerate() {
                println!(
                    "Sensor {}: min => {:.06}, max => {:.06}, avg => {:.06}",
                    i,
                    sensor.min(),
                    sensor.max(),
                    sensor.avg()
                );
            }
            println!("After reading: {:?}", locked_buffer.metadata);
        }
        if !args.nowait {
            std::thread::sleep(std::time::Duration::from_millis(10_000));
        }
    }
}

fn main() {
    let args = Args::parse();
    let data = Arc::new(Mutex::new(SensorsBuffer {
        buffer: vec![SensorData::default(); args.samples as usize],
        metadata: SensorFileMetadata::from_size(args.samples),
    }));
    let data_prod = Arc::clone(&data);
    let args_prod = args.clone();
    let consumer_thread = std::thread::spawn(move || {
        consumer(data_prod, args_prod);
    });

    let data_cons = data;
    let args_cons = args;
    let producer_thread = std::thread::spawn(move || {
        producer(data_cons, args_cons);
    });

    consumer_thread.join().unwrap();
    producer_thread.join().unwrap();
}
