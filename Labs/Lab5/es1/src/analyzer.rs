use std::{io::{BufReader, BufRead}, fs::File, collections::HashMap};

fn main() {
    let bufr = BufReader::new(std::io::stdin());
    for path in bufr.lines() {
        let mut map = HashMap::new();
        let path = match path {
            Ok(path) => path,
            Err(_) => {
                println!("{{}}");
                return;
            }
        };
        let f = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                println!("{{}}");
                return;
            }
        };
        let filereader = BufReader::new(f);
        for line in filereader.lines() {
            let line = line.unwrap_or_else(|_| "".to_string());
            let words = line.split_whitespace();
            for word in words {
                let count = map.entry(word.to_string()).or_insert(0);
                *count += 1;
            }
        }
        println!("{}", serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string()));
    }
}