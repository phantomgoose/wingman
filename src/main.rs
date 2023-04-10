use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, stdout, Write};

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
    let client = Client::new();

    let user_id = format!("wingman-user-{}", Uuid::new_v4());

    let mut stdout = stdout();

    // continually prompt for input until the user types "quit"
    let mut prompt;

    // store the last 10 prompts
    // TODO: be smarter about this, maybe just the tokens rather than the whole prompt to be able to avoid hitting token limits?
    let max_items = 10;
    let mut prompt_store: VecDeque<PromptResponse> = VecDeque::with_capacity(max_items);

    loop {
        println!("Enter your prompt (type 'quit' or press ^C to exit):");

        // clear the prompt
        prompt = String::new();
        // read the prompt from the user
        stdin().read_line(&mut prompt)?;

        if prompt.trim().to_lowercase().eq("quit") {
            println!("Exiting Wingman session...");
            break;
        }

        if prompt.trim().len() < 1 {
            println!("Prompt must be at least 1 character long.");
            continue;
        }

        let previous_promps = prompt_store
            .iter()
            .map(|prompt_response| {
                format!(
                    "Prompt: {}\nResponse: {}\n",
                    prompt_response.prompt, prompt_response.response
                )
            })
            .collect::<Vec<String>>();

        let current_prompt = format!(
            "Prompt: {}\n. Do not prefix your response with the word \"Response\"",
            prompt
        );

        let combined_prompts = previous_promps.join("") + &current_prompt;

        // send the prompt to OpenAI
        let request = CreateCompletionRequestArgs::default()
            .model("text-davinci-003")
            .prompt(&combined_prompts)
            .max_tokens(2048u16)
            // generate a random unique ID for this session
            .user(&user_id)
            .temperature(0.9)
            .n(1)
            .stream(true)
            .stop("\u{0}")
            .build()
            .unwrap();

        let response = client.completions().create_stream(request).await?;

        let mut response_str = String::new();
        response
            .for_each(|response| {
                let token = response
                    .unwrap()
                    .choices
                    .iter()
                    .map(|choice| choice.text.as_str())
                    .collect::<Vec<&str>>()
                    .join("");
                print!("{}", token);
                stdout.flush().unwrap();

                response_str.push_str(token.as_str());
                future::ready(())
            })
            .await;

        while prompt_store.len() >= max_items {
            prompt_store.pop_front();
        }

        prompt_store.push_back(PromptResponse {
            prompt: prompt.clone(),
            response: response_str.clone(),
        });

        // insert empty line
        println!();
    }

    Ok(())
}
