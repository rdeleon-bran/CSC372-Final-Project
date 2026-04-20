use std::env;
use std::process;

fn main() {
    // collect() gathers all command-line arguments into a list (Vec)
    // args[0] is the program name, args[1] is what the user types
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: {} <image.pgm>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    println!("Input file: {}", input_path);
}

