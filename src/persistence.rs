use std::collections::VecDeque;

use async_openai::types::{ChatCompletionRequestMessage, Role};

const MAX_HISTORICAL_MESSAGES: usize = 20;

pub fn store_message(
    message_history: &mut VecDeque<ChatCompletionRequestMessage>,
    content: String,
    role: Role,
) {
    // tidy up the prompt store and push the most recent interaction to it
    while message_history.len() >= MAX_HISTORICAL_MESSAGES {
        message_history.pop_front();
    }

    message_history.push_back(ChatCompletionRequestMessage {
        role,
        content,
        name: None,
    });
}
