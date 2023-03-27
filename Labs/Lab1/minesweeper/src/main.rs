use clap::Parser;
use minesweeper::{annotate, annotate2};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    rows: usize,
    #[arg(short, long)]
    cols: usize,
    board: String,
}

/*
fn transform_board(board: &str, cols: usize) -> Vec<&str> {
    // unsafe because we are assuming that the board is valid utf8
    board.as_bytes().chunks(cols).map(|c| unsafe { std::str::from_utf8_unchecked(c) }).collect::<Vec<&str>>()
}
*/
fn transform_board(rows: usize, cols: usize, input_board: &str) -> Vec<&str> {
    let mut board = vec![];
    for i in 0..rows {
        let start_idx = i * cols;
        let end_idx = start_idx + cols;
        let row = &input_board[start_idx..end_idx];
        board.push(row);
    }
    board
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let board = transform_board(args.rows, args.cols, &args.board);
    let result = annotate(&board);
    let result_single_string = annotate2(args.board, args.rows, args.cols)?;
    let transformed_result = transform_board(args.rows, args.cols, &result_single_string);
    println!("***Result with annotate2: ***");
    for row in transformed_result {
        println!("{}", row);
    }
    println!("***Result with annotate : ***");
    for row in result {
        println!("{}", row);
    }
    Ok(())
}
