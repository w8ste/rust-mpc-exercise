use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::thread;

use crate::circuit::circuit_parser::Circuit;
use crate::party::party_gmw::new_party_pair;

pub mod circuit;
pub mod mul_triple;
pub mod party;

/// For argument parsing, my favorite crate is clap https://docs.rs/clap/latest/clap/
/// Especially its derive feature makes declarative argument parsing really easy.
/// You can add clap as a dependency with the derive feature and annotate this struct
/// and add the necessary fields.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to file, which contains circuit in bristol fashion
    #[arg(short, long)]
    path: PathBuf,
    /// Input for party 0
    #[arg(short, long)]
    first_in: u64,
    /// Input for party 1
    #[arg(short, long)]
    second_in: u64,
}

fn main() {
    // The main function should first parse the passed arguments (I recommend to use a crate like
    // clap), and then evaluate the passed circuit. Note that you will likely need to run each
    // Party in its own thread (see https://doc.rust-lang.org/std/thread/index.html).
    let args = Args::parse();
    let filepath = args.path;
    let file_contents: String = match fs::read_to_string(filepath) {
        Ok(contents) => contents,
        Err(e) => {
            // print error message and exit from the program
            eprintln!("An error has occurred whilst accessing the file: {}!", e);
            std::process::exit(1);
        }
    };

    let c: Circuit = match Circuit::parse(&file_contents) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let (mut p0, mut p1) = new_party_pair(c);

    let first: u64 = args.first_in;
    let second: u64 = args.second_in;

    let mut input_p0 = [false; 64];
    let mut input_p1 = [false; 64];

    for i in 0..64 {
        input_p0[i] = (first >> i) & 1 == 1;
        input_p1[i] = (second >> i) & 1 == 1;
    }

    let p0 = thread::spawn(move || p0.execute(&input_p0).unwrap());
    let p1 = thread::spawn(move || p1.execute(&input_p1).unwrap());

    let sol_p0 = match p0.join() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error occurred while joining p0 thread: {:?}", e);
            std::process::exit(1);
        }
    };

    let sol_p1 = match p1.join() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error occurred while joining p1 thread: {:?}", e);
            std::process::exit(1);
        }
    };

    assert_eq!(sol_p0, sol_p1);

    let mut solution: i64 = 0;
    for (i, v) in sol_p0.iter().enumerate().take(64) {
        solution += if *v { 1 } else { 0 } << i;
    }

    println!("The result of the calculation is {}", solution)
}
