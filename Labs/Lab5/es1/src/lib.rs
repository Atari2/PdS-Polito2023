use clap::Parser;
#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub directory: String
}