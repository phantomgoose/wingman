use std::error::Error;

use async_openai::{
    types::{CreateImageRequestArgs, ImageSize, ResponseFormat},
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create client, reads OPENAI_API_KEY environment variable for API key.
    let client = Client::new();

    let request = CreateImageRequestArgs::default()
        .prompt("cats on sofa and carpet in living room")
        .n(2)
        .response_format(ResponseFormat::Url)
        .size(ImageSize::S256x256)
        .build()?;

    let response = client.images().create(request).await?;

    // Download and save images to ./data directory.
    // Each url is downloaded and saved in dedicated Tokio task.
    // Directory is created if it doesn't exist.
    let paths = response.save("./out").await?;

    paths
        .iter()
        .for_each(|path| println!("Image file path: {}", path.display()));

    Ok(())
}
