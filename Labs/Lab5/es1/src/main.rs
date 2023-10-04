use std::{collections::HashMap, process::{Command, Stdio, Child, ChildStdin, ChildStdout}, io::{Write, BufReader, BufRead}, fs::File};

use clap::Parser;
use es1::Args;
use walkdir::WalkDir;

type GenericRes<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct ProcessPool {
    workers: Vec<Child>
}

impl ProcessPool {
    fn new(procs: usize) -> GenericRes<ProcessPool> {
        let mut workers = Vec::new();
        for _ in 0..procs {
            let child = Command::new("analyzer").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
            workers.push(child);
        }
        Ok(ProcessPool { workers })
    }
    fn extract_stdins(&mut self) -> GenericRes<Vec<ChildStdin>> {
        let mut stdins = Vec::new();
        for worker in self.workers.iter_mut() {        
            let stdin = worker.stdin.take().ok_or("Failed to take stdin")?;
            stdins.push(stdin);
        }
        Ok(stdins)
    }
    fn extract_stdouts(&mut self) -> GenericRes<Vec<BufReader<ChildStdout>>> {
        let mut stdouts = Vec::new();
        for worker in self.workers.iter_mut() {
            let stdout = worker.stdout.take().ok_or("Failed to take stdout")?;
            stdouts.push(BufReader::new(stdout));
        }
        Ok(stdouts)
    }
}

fn main() -> GenericRes<()> {
    const NPROCS: usize = 5;
    let args = Args::parse();
    let mut procpool = ProcessPool::new(NPROCS)?;
    let mut stdins = procpool.extract_stdins()?;
    let mut stdouts = procpool.extract_stdouts()?;
    let feeder = std::thread::spawn(move || -> GenericRes<()> {
        let mut n = 0;
        for entry in WalkDir::new(args.directory).into_iter() {
            let p = entry?;        
            if p.path().extension().map_or(false, |ext| ext == "txt") {
                stdins[n % NPROCS].write_all(format!("{}\n", p.path().to_str().ok_or("Invalid path")?).as_bytes())?;
                n += 1;
            }
        }
        Ok(())
    });
    let receiver = std::thread::spawn(move || -> GenericRes<HashMap<String, u32>> {
        let mut n = 0;
        let mut res_map: HashMap<String, u32> = HashMap::new();
        loop {
            let mut buf = String::new();
            let res = stdouts[n % NPROCS].read_line(&mut buf);
            match res {
                Ok(0) => break,
                Ok(_) => {
                    let map: HashMap<String, u32> = serde_json::from_str(&buf)?;
                    for (key, value) in map {
                        let count = res_map.entry(key).or_insert(0);
                        *count += value;
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
            }
            n += 1;
        }
        Ok(res_map)
    });
    match feeder.join() {
        Ok(res) => res?,
        Err(e) => {
            println!("Error: {:?}", e);
            return Ok(())
        }
    }
    let res_map = match receiver.join() {
        Ok(res) => res?,
        Err(e) => {
            println!("Error: {:?}", e);
            return Ok(())
        }
    };
    File::create("result.json")?.write_all(serde_json::to_string_pretty(&res_map)?.as_bytes())?;
    Ok(())
}