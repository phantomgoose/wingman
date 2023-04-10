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

#[cfg(test)]
mod tests {
    use super::is_exit_code;

    fn test_is_exit_code(input: &str, expected: bool) {
        assert_eq!(is_exit_code(input), expected);
        assert_eq!(is_exit_code(input.to_uppercase().as_str()), expected);
        assert_eq!(is_exit_code(input.to_lowercase().as_str()), expected);
    }

    #[test]
    fn test_is_exit_code_q() {
        test_is_exit_code("q", true);
    }

    #[test]
    fn test_is_exit_code_quit() {
        test_is_exit_code("quit", true);
    }

    #[test]
    fn test_is_exit_code_random() {
        test_is_exit_code("random", false);
    }
}
