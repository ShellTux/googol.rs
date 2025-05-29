use std::env;
use std::fs::File;
use std::process;

fn main() {
    // Collect command-line arguments into a vector
    let args: Vec<String> = env::args().collect();

    // Check if at least one argument is provided (excluding program name)
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    // Get the first argument (filename)
    let filename = &args[1];

    // Create an empty file with the specified filename
    match File::create(filename) {
        Ok(_) => println!("Created file: {}", filename),
        Err(e) => {
            eprintln!("Failed to create file {}: {}", filename, e);
            process::exit(1);
        }
    }
}
