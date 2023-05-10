use std::collections::HashMap;
pub fn frequency(input: &[&str], worker_count: usize) -> HashMap<char, usize> {
    let mut handles = vec![];
    if input.is_empty() {
        return HashMap::new();
    }
    let chunk_size = match input.len() / worker_count {
        0 => 1,
        n => n,
    };
    for chunk in input.chunks(chunk_size) {
        if chunk.is_empty() {
            continue;
        }
        // TODO: Find a way to avoid this Vec<String> allocation
        let chunk = chunk.iter().map(|&s| s.to_string()).collect::<Vec<String>>();
        let handle = std::thread::spawn(move || {
            let mut map = HashMap::new();
            for word in chunk {
                for c in word.chars().filter(|c| c.is_alphabetic()).flat_map(|c| c.to_lowercase()) {
                    let counter = map.entry(c).or_insert(0);
                    *counter += 1;
                }
            }
            map
        });
        handles.push(handle);
    }
    let mut map = HashMap::new();
    for handle in handles {
        let intermediate = handle.join().unwrap();
        for (k, v) in intermediate.into_iter() {
            let counter = map.entry(k).or_insert(0);
            *counter += v;
        }
    }
    map
}
