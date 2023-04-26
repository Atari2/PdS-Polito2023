use clap::Parser;
use es3::{Args, Calendar};

fn main() {
    let args = Args::parse();
    let cal1 = match Calendar::from_file(args.cal1) {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };
    let cal2 = match Calendar::from_file(args.cal2) {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };
    for (start, end) in cal1
        .find_slots(args.duration)
        .chain(cal2.find_slots(args.duration))
    {
        println!("{} {}", start, end);
    }
}
