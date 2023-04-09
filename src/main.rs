use std::error::Error;

use async_openai::{
    types::{CreateCompletionRequestArgs, ImageSize, ResponseFormat},
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create client, reads OPENAI_API_KEY environment variable for API key.
    let client = Client::new();

    let request = CreateCompletionRequestArgs::default()
        .model("text-davinci-003") // TODO: this is not the right model
        .prompt("Say hello and tell me a joke")
        .temperature(0.9)
        .n(1)
        // .stream(false) -- try this out later
        .stop("\u{0}")
        .best_of(1)
        .build()
        .unwrap();

    let response = client.completions().create(request).await?;

    println!("{:#?}", response.choices[0].text);

    Ok(())
}
