use std::collections::VecDeque;
use std::error::Error;

use async_openai::types::ChatCompletionRequestMessage;
use async_openai::types::Role::{System, User};
use async_openai::Client;
use uuid::Uuid;

use crate::chat::process_chat_message;
use crate::persistence::store_message;
use crate::terminal::{get_new_prompt, print_streaming_response, PromptStatus};

mod chat;
mod persistence;
mod terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Wingman session...");

    // OpenAI client uses the OPENAI_API_KEY environment variable as its API key
    let open_ai_client = Client::new();
    // generate a random unique ID for this session
    let user_id = &format!("wingman-user-{}", Uuid::new_v4());

    let mut message_history: VecDeque<ChatCompletionRequestMessage> = VecDeque::new();

    println!("Enter your prompt (type 'q', 'quit,' or press ^C to exit):");
    loop {
        let trimmed_prompt = match get_new_prompt() {
            PromptStatus::Success(prompt) => prompt,
            PromptStatus::Exit => {
                break;
            }
        };

        store_message(&mut message_history, trimmed_prompt.to_string(), User);

        let response_str = process_chat_message(
            &open_ai_client,
            print_streaming_response,
            message_history.clone(),
            user_id,
        )
        .await
        .expect("Failed to process chat message");

        store_message(&mut message_history, response_str, System);

        println!("\nPrompt: ");
    }

    Ok(())
}
