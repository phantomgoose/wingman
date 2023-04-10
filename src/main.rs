use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, stdout, Write};

use async_openai::error::OpenAIError::ApiError;
use async_openai::types::Role::{System, User};
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs};
use async_openai::Client;
use futures::{future, StreamExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Wingman session...");

    // create client, reads OPENAI_API_KEY environment variable for API key.
    let open_ai_client = Client::new();
    // generate a random unique ID for this session
    let user_id = format!("wingman-user-{}", Uuid::new_v4());
    let mut prompt;

    const MAX_HISTORICAL_MESSAGES: usize = 20;
    let mut message_history: VecDeque<ChatCompletionRequestMessage> = VecDeque::new();

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

        message_history.push_back(ChatCompletionRequestMessage {
            role: User,
            content: trimmed_prompt.to_string(),
            name: None,
        });

        let prompt_to_send = message_history.clone();

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4")
            .messages(prompt_to_send)
            .max_tokens(2048u16)
            .user(&user_id)
            .temperature(0.9)
            .n(1)
            .stream(true)
            .stop("\u{0}")
            .build()
            .expect("Failed to build OpenAI request");

        let response = open_ai_client.chat().create_stream(request).await?;

        print!("Response: ");
        let mut response_str = String::new();
        response
            .for_each(|response| {
                match response {
                    Ok(response) => {
                        let token = response
                            .choices
                            .iter()
                            .map(|choice| choice.delta.content.clone().unwrap_or_else(|| "".to_string()))
                            .collect::<Vec<String>>()
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
        while message_history.len() >= MAX_HISTORICAL_MESSAGES {
            message_history.pop_front();
        }

        message_history.push_back(ChatCompletionRequestMessage {
            role: System,
            content: response_str,
            name: None,
        });

        println!("\nPrompt: ");
    }

    Ok(())
}
