
Steps to Create the Project Locally:

1. Create a Project Directory: Open your terminal or command prompt and run:

mkdir lm-aichat
cd lm-aichat
cargo init

This will create a new Rust project with the following structure:

lm-aichat/
├── src/
│   ├── main.rs
├── Cargo.toml


2. Add Dependencies to Cargo.toml: Open Cargo.toml and replace the content with the following:

[package]
name = "lm-aichat"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
tokio-tungstenite = "0.23.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures-util = "0.3"
url = "2.2"

# aichat and lm.rs dependencies
wide = "0.7.28"
memmap2 = "0.9.4"
rayon = "1.10.0"
chrono = "0.4.38"
clap = { version = "4.5.13", features = ["derive"] }


3. Create backend.rs: In the src folder, create a new file named backend.rs and paste the following code:

use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use serde_json::json;
use tokio::task;

async fn handle_connection(raw_stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(raw_stream).await.expect("Error during the websocket handshake");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response = json!({
                    "embedding": [0.1, 0.5, 0.9],
                    "message": format!("Processed: {}", text),
                });

                write.send(Message::Text(response.to_string())).await.expect("Failed to send message");
            }
            Ok(Message::Close(_)) => break,
            _ => (),
        }
    }
}

pub async fn start_backend_server() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Cannot bind to address");

    println!("Backend running on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(handle_connection(stream));
    }
}


4. Create aichat_client.rs: Create another file in the src folder named aichat_client.rs with the following content:

use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;

pub async fn start_aichat_client(input_message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = url::Url::parse("ws://127.0.0.1:8080")?;
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(input_message.into())).await?;

    if let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(msg))) = ws_stream.next().await {
        let parsed: Value = serde_json::from_str(&msg)?;
        let embedding = &parsed["embedding"];
        let processed_message = &parsed["message"];

        println!("Received from lm.rs:");
        println!("Embedding: {:?}", embedding);
        println!("Processed Message: {}", processed_message);
    }

    Ok(())
}


5. Modify main.rs: In the existing src/main.rs, replace its content with:

mod backend;
mod aichat_client;

use tokio::task;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let backend_task = task::spawn(async {
        backend::start_backend_server().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_task = task::spawn(async {
        let input = "Hello from aichat!";
        if let Err(e) = aichat_client::start_aichat_client(input).await {
            eprintln!("Error in client: {}", e);
        }
    });

    backend_task.await?;
    client_task.await?;

    Ok(())
}


6. Build and Run the Project: After saving all the files, you can build and run the project by running:

cargo run

This will start both the lm.rs backend server and the aichat client. The client will send a message to the backend and receive the processed response (e.g., embeddings).



Conclusion:

By following these steps, you'll have a fully functional project that integrates lm.rs with aichat, allowing communication between a backend server (processing embeddings) and a client generating responses.

