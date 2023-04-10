use std::error::Error;
use std::io::stdin;
use std::io::{stdout, Write};

use async_openai::{types::CreateCompletionRequestArgs, Client};
use crossterm::{
    cursor::{MoveToColumn, MoveToPreviousLine, RestorePosition, SavePosition},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use futures::{future, stream, StreamExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Wingman session...");

    // create client, reads OPENAI_API_KEY environment variable for API key.
    let client = Client::new();

    let user_id = format!("wingman-user-{}", Uuid::new_v4());

    let mut stdout = stdout();

    // continually prompt for input until the user types "quit"
    let mut prompt = String::new();

    loop {
        println!("Enter your prompt (type 'quit' to exit):");

        // clear the prompt
        prompt = String::new();
        // read the prompt from the user
        stdin().read_line(&mut prompt)?;

        if prompt.trim().eq("quit") {
            println!("Exiting Wingman session...");
            break;
        }

        println!("Sending request to OpenAI: {}", &prompt);

        // send the prompt to OpenAI
        let request = CreateCompletionRequestArgs::default()
            .model("text-davinci-003")
            .prompt(&prompt)
            // generate a random unique ID for this session
            .user(&user_id)
            .temperature(0.9)
            .n(1)
            .stream(true)
            .stop("\u{0}")
            .build()
            .unwrap();

        let response = client.completions().create_stream(request).await?;

        println!("Received response from OpenAI, reading...");

        let mut response_str = String::new();
        response
            .for_each(|response| {
                response_str.push_str(
                    response
                        .unwrap()
                        .choices
                        .iter()
                        .map(|choice| choice.text.as_str())
                        .collect::<Vec<&str>>()
                        .join("")
                        .as_str(),
                );
                // clear the line and print the response
                execute!(
                    stdout,
                    MoveToPreviousLine(1),
                    Clear(ClearType::CurrentLine),
                    Print(&response_str)
                )
                .unwrap();
                future::ready(())
            })
            .await;
    }

    Ok(())
}
