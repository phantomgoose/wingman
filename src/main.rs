use std::io::{self, BufRead, Write};

fn main() {
    // Display a prompt for user input
    print!("Enter your input: ");
    io::stdout().flush().expect("Failed to flush stdout");

    // Read user input from stdin
    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_line(&mut input)
        .expect("Failed to read from stdin");

    // Remove the trailing newline character
    input.pop();

    // Print the user input back to the console
    println!("You entered: {}", input);
}
