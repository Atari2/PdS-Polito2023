use clap::Parser;
use luhn::is_valid;

#[derive(Parser)]
struct Args {
    input: String
}

fn pre_validation(code: &str) -> bool {
    let partials = code.split_whitespace().collect::<Vec<&str>>();
    if partials.len() != 4 {
        return false;
    } else {
        for partial in partials {
            if partial.len() != 4 || !partial.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
    }
    true
}

fn main() {
    let args = Args::parse();
    if !pre_validation(&args.input) {
        println!("{} is invalid (failed pre-validation)", args.input);
        return;
    }
    if is_valid(&args.input) {
        println!("{} is valid", args.input);
    } else {
        println!("{} is invalid", args.input);
    }
}