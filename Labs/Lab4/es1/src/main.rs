use std::{fmt::Display, time::Instant};

use clap::Parser;
use itertools::Itertools;
use std::sync::Arc;

#[derive(Debug)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    fn apply(&self, a: i32, b: i32) -> i32 {
        match self {
            Operation::Add => a + b,
            Operation::Sub => a - b,
            Operation::Mul => a * b,
            Operation::Div => a / b,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            Operation::Add => "+",
            Operation::Sub => "-",
            Operation::Mul => "*",
            Operation::Div => "/",
        };
        write!(f, "{}", op)
    }
}

fn check_solution(np: &[&i32], op: &[&Operation], aim: i32) -> Option<String> {
    let mut res = *np[0];
    for (i, o) in op.iter().enumerate() {
        res = o.apply(res, *np[i + 1]);
    }
    if res == aim {
        Some(format!(
            "{} {} {} {} {} {} {} {} {} = {}",
            np[0], op[0], np[1], op[1], np[2], op[2], np[3], op[3], np[4], aim
        ))
    } else {
        None
    }
}

fn check_solution_threaded(np: &[i32], op: &[&Operation], aim: i32) -> Option<String> {
    let mut res = np[0];
    for (i, o) in op.iter().enumerate() {
        res = o.apply(res, np[i + 1]);
    }
    if res == aim {
        Some(format!(
            "{} {} {} {} {} {} {} {} {} = {}",
            np[0], op[0], np[1], op[1], np[2], op[2], np[3], op[3], np[4], aim
        ))
    } else {
        None
    }
}

fn solve(numbers: [i32; 5]) -> Vec<String> {
    const SUM_AIM: i32 = 10;
    const OPS: [Operation; 4] = [
        Operation::Add,
        Operation::Sub,
        Operation::Mul,
        Operation::Div,
    ];
    let mut result = Vec::new();
    for np in numbers.iter().permutations(5) {
        for op in OPS.iter().combinations_with_replacement(4) {
            if let Some(line) = check_solution(&np, &op, SUM_AIM) {
                result.push(line);
            }
        }
    }
    result
}

fn solve_with_thread(numbers: [i32; 5], n_threads: usize) -> Vec<String> {
    const SUM_AIM: i32 = 10;
    const OPS: [Operation; 4] = [
        Operation::Add,
        Operation::Sub,
        Operation::Mul,
        Operation::Div,
    ];
    let mut result = vec![];
    let perms = Arc::new(numbers.into_iter().permutations(5).collect::<Vec<_>>());
    let nperms = perms.len() / n_threads;
    let mut tids = vec![];
    for i in 0..n_threads {
        let start = i * nperms;
        let mut end = start + nperms;
        if i == n_threads - 1 {
            end = perms.len();
        }
        let perms = Arc::clone(&perms);
        let t = std::thread::spawn(move || {
            let mut res = vec![];
            let np = &perms[start..end];
            for np in np {
                for op in OPS.iter().combinations_with_replacement(4) {
                    if let Some(line) = check_solution_threaded(np, &op, SUM_AIM) {
                        res.push(line);
                    }
                }
            }
            res
        });
        tids.push(t);
    }
    for t in tids {
        let mut res = t.join().unwrap();
        result.append(&mut res);
    }
    result
}

#[derive(Parser)]
struct Numbers {
    n1: i32,
    n2: i32,
    n3: i32,
    n4: i32,
    n5: i32,
    #[clap(short, long)]
    n_threads: Option<usize>,
}

#[test]
fn test_solve() {
    let example_numbers = [1, 2, 3, 4, 5];
    let result = solve(example_numbers);
    let result2 = solve_with_thread(example_numbers, 4);
    assert_eq!(result, result2);
}

fn main() {
    let numbers = Numbers::parse();
    let nums = [numbers.n1, numbers.n2, numbers.n3, numbers.n4, numbers.n5];
    let valid_range = 0..=9;
    if nums.iter().any(|&n| !valid_range.contains(&n)) {
        println!("Invalid input, all numbers must be between 0 and 9");
    }
    let result;
    let start = Instant::now();
    if let Some(n) = numbers.n_threads {
        if n == 1 {
            result = solve(nums);
        } else {
            result = solve_with_thread(nums, n);
        }
    } else {
        result = solve(nums);
    }
    let elapsed = start.elapsed();
    for line in result {
        println!("{}", line);
    }
    println!("Time elapsed (with {:?} threads): {:?}", numbers.n_threads, elapsed);
}
