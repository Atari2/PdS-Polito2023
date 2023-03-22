use clap::Parser;
/*
La funzione deve convertire in maiuscolo il primo carattere di ogni parola che compone il 
testo s, ignorando eventuali altri caratteri maiuscoli al suo interno.
Le parole sono separate da uno o più spazi
*/

fn capitalize(s: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for c in s.chars() {
        if first {
            result.push_str(c.to_uppercase().to_string().as_str());
            first = false;
        } else {
            first = c.is_whitespace();
            result.push(c);
        }
    }
    result
}

#[derive(Parser, Debug)]
struct Args {
    input: String,
}

fn main() {
    let args = Args::parse();
    println!("{}", capitalize(&args.input));
}
