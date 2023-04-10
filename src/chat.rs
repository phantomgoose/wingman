use std::collections::VecDeque;

use async_openai::error::OpenAIError::ApiError;
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionRequestArgs,
    CreateChatCompletionStreamResponse,
};
use async_openai::Client;
use futures::future;
use futures::StreamExt;

fn get_token_from_response(response: CreateChatCompletionStreamResponse) -> String {
    response
        .choices
        .iter()
        .map(|choice| {
            choice
                .delta
                .content
                .clone()
                .unwrap_or_else(|| "".to_string())
        })
        .collect::<Vec<String>>()
        .join("")
}

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
                    let token = get_token_from_response(response);
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

#[cfg(test)]
mod tests {
    use async_openai::types::{
        ChatChoiceDelta, ChatCompletionResponseStreamMessage, CreateChatCompletionStreamResponse,
    };

    use super::get_token_from_response;

    fn test_get_token_from_response(content: String, expected: String) {
        let response = CreateChatCompletionStreamResponse {
            choices: vec![ChatChoiceDelta {
                index: 0,
                delta: ChatCompletionResponseStreamMessage {
                    content: Some(content),
                    role: None,
                },
                finish_reason: None,
            }],
            created: 0,
            id: Some("".to_string()),
            model: "".to_string(),
            object: "".to_string(),
            usage: None,
        };

        let token = get_token_from_response(response);

        assert_eq!(token, expected);
    }

    #[test]
    fn test_get_token_from_response_with_content() {
        test_get_token_from_response("Hello world!".to_string(), "Hello world!".to_string());
    }
}
