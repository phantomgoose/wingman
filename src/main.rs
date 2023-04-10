use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, stdout, Write};

use async_openai::error::OpenAIError::ApiError;
use async_openai::{types::CreateCompletionRequestArgs, Client};
use futures::{future, StreamExt};
use uuid::Uuid;

struct PromptResponse {
    prompt: String,
    response: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Wingman session...");

    // create client, reads OPENAI_API_KEY environment variable for API key.
    let open_ai_client = Client::new();
    let user_id = format!("wingman-user-{}", Uuid::new_v4());
    let mut prompt;

    const MAX_PROMPTS_TO_STORE: usize = 10;
    let mut prompt_store: VecDeque<PromptResponse> = VecDeque::with_capacity(MAX_PROMPTS_TO_STORE);

    println!("Enter your prompt (type 'q', 'quit,' or press ^C to exit):");
    loop {
        prompt = String::new();
        stdin().read_line(&mut prompt)?;

        let trimmed_prompt = prompt.trim();
        if trimmed_prompt.eq_ignore_ascii_case("q") || trimmed_prompt.eq_ignore_ascii_case("quit") {
            println!("Exiting Wingman session...");
            break;
        }

        if trimmed_prompt.is_empty() {
            println!("Prompt must be at least 1 character long.");
            continue;
        }

        let previous_prompts = prompt_store
            .iter()
            .map(|prompt_response| {
                format!(
                    "Prompt: {}\nResponse: {}\n",
                    prompt_response.prompt, prompt_response.response
                )
            })
            .collect::<Vec<String>>();

        let current_prompt = format!(
            "Prompt: {}\n. Do not start your response with a newline. Do not prefix your response with the word \"Response\", a newline character, or any other word or character.",
            prompt
        );

        let prompts_to_send = previous_prompts.join("") + &current_prompt;

        let request = CreateCompletionRequestArgs::default()
            .model("text-davinci-003")
            .prompt(&prompts_to_send)
            .max_tokens(2048u16)
            // generate a random unique ID for this session
            .user(&user_id)
            .temperature(0.5)
            .n(1)
            .stream(true)
            .stop("\u{0}")
            .build()
            .expect("Failed to build OpenAI request");

        let response = open_ai_client.completions().create_stream(request).await?;

        print!("Response: ");
        let mut response_str = String::new();
        response
            .for_each(|response| {
                match response {
                    Ok(response) => {
                        let token = response
                            .choices
                            .iter()
                            .map(|choice| choice.text.as_str())
                            .collect::<Vec<&str>>()
                            .join("");
                        print!("{}", token);
                        stdout().flush().expect("Failed to flush stdout");

                        response_str.push_str(&token);
                    }
                    Err(err) => {
                        match err {
                            ApiError(err) => {
                                // assume API errors are transient
                                println!(
                                    "Received an error from OpenAI: {}",
                                    format!("{} {} {}", err.code.unwrap(), err.r#type, err.message)
                                );
                            }
                            _ => {
                                if err.to_string().contains("401 Unauthorized") {
                                    panic!("OpenAI API key is invalid. Please check your OPENAI_API_KEY environment variable.")
                                }
                                panic!("{}", err);
                            }
                        }
                    }
                }

                future::ready(())
            })
            .await;

        // tidy up the prompt store and push the most recent interaction to it
        if prompt_store.len() >= MAX_PROMPTS_TO_STORE {
            prompt_store.pop_front();
        }

        prompt_store.push_back(PromptResponse {
            prompt: prompt.clone(),
            response: response_str.clone(),
        });

        println!("\nPrompt: ");
    }

    Ok(())
}
