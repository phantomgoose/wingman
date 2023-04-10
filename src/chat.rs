use std::collections::VecDeque;

use async_openai::error::OpenAIError::ApiError;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs};
use async_openai::Client;
use futures::future;
use futures::StreamExt;

pub async fn process_chat_message(
    open_ai_client: &Client,
    on_new_msg: fn(token: &String),
    messages: VecDeque<ChatCompletionRequestMessage>,
    user_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages(messages)
        .max_tokens(2048u16)
        .user(user_id)
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
                    on_new_msg(&token);

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
        }).await;
    Ok(response_str)
}
