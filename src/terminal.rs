use std::io::{stdin, stdout, Write};

#[derive(Debug)]
pub enum PromptStatus {
    Success(String),
    Exit,
}

fn is_exit_code(trimmed_prompt: &str) -> bool {
    trimmed_prompt.eq_ignore_ascii_case("q") || trimmed_prompt.eq_ignore_ascii_case("quit")
}

pub fn get_new_prompt() -> PromptStatus {
    loop {
        let mut prompt = String::new();
        stdin().read_line(&mut prompt).expect("Failed to read line");

        let trimmed_prompt = prompt.trim();
        if is_exit_code(trimmed_prompt) {
            println!("Exiting Wingman session...");
            return PromptStatus::Exit;
        }

        if trimmed_prompt.is_empty() {
            println!("Prompt must be at least 1 character long.");
            continue;
        }

        return PromptStatus::Success(trimmed_prompt.to_string());
    }
}

pub fn print_streaming_response(token: &String) {
    print!("{}", token);
    stdout().flush().unwrap();
}
