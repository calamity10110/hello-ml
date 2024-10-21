
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


6. Build and Run the Project: 
Install additional python dependencies (assuming you already have pytorch installed) used in export.py and tokenizer.py:

pip install -r requirements.txt
Download the .safetensors and config.json files from the original model's page on huggingface (So we don't have to clone the pytorch repo). For multimodal models (PHI3.5 Vision), we also need the CLIP .config file.

Use the export.py script to convert the model bfloat16 weights into the LMRS format:

python export.py --files [ordered .safetensor files] --config [model config.json] --save-path [name and path to save] --type [model type (GEMMA/LLAMA/PHI)]
To export the quantized version use the --quantize and --quantize-type flags. The int8 quantized model size should be 4X smaller (from ~9.8G to ~2.5G, depending on the group size). For multimodal models include the --vision-config argument.

Use the tokenizer.py script to convert the tokenizer model into the LMRS tokenizer format:

python tokenizer.py --model-id [huggingface model_id] --tokenizer-type [type of the tokenizer (GEMMA/LLAMA/PHI)]

Build

Compile the rust code with cargo (make sure to pass the target-cpu flag):

To run the backend for the WebUI, first compile:

RUSTFLAGS="-C target-cpu=native" cargo build --release --features all --bin backend

then start the model by running:
./target/release/backend --model [model weights file]



cargo run

This will start both the lm.rs backend server and the aichat client. The client will send a message to the backend and receive the processed response (e.g., embeddings).



🌃 Now supporting multimodality with PHI-3.5-vision model! PHI-3.5-mini text-only model also now supported.

Inspired by Karpathy's llama2.c and llm.c I decided to create the most minimal code (not so minimal atm) that can perform full inference on Language Models on the CPU without ML libraries. Previously only Google's Gemma 2 models were supported, but I decided to add support for the new Llama 3.2 models, and more recently the option to use images with PHI-3.5.

News: Implemented batch processing, boosting the image encoding speed by up to ~3x. Llama 3.2 1B now runs at 50 tok/s on my 16-core machine.

Disclaimer: Some of the code could be optimized and improved. This is just an excuse for me to write Rust for the first time. Isn't it incredible that in a few years, we could have AGI running in a few lines of poorly written Rust code?

Prepared models
Some benchmarks and download links for the models and tokenizers. I recommend using Q8_0, Q4_0 quantization still being improved. Speed measured on a 16-core AMD Epyc.

Model	Size	Speed
Gemma 2 2B IT Q4_0	1.39G	20 tok/s
Gemma 2 2B IT Q8_0	2.66GB	24 tok/s
Gemma 2 9B IT Q4_0	4.91GB	7 tok/s
Gemma 2 9B IT Q8_0	9.53GB	8 tok/s
Llama 3.2 1B IT	4.94GB	21 tok/s
Llama 3.2 1B IT Q8_0	1.27GB	50 tok/s
Llama 3.2 3B IT Q4_0	1.71GB	17 tok/s
Llama 3.2 3B IT Q8_0	3.31GB	19 tok/s
PHI 3.5 IT Vision Q8_0	4.28GB	17 tok/s
PHI 3.5 IT Mini Q8_0	3.94GB	18 tok/s

Instructions for starting lm.rs

You can download the prepared quantized model and tokenizer model files in the lmrs format from huggingface. If you'd prefer to convert the models published by Google/Meta on huggingface yourself, please refer to the following section. Otherwise, you can skip ahead to the build section.

Model Conversion
Install additional python dependencies (assuming you already have pytorch installed) used in export.py and tokenizer.py:

pip install -r requirements.txt
Download the .safetensors and config.json files from the original model's page on huggingface (So we don't have to clone the pytorch repo). For multimodal models (PHI3.5 Vision), we also need the CLIP .config file.

Use the export.py script to convert the model bfloat16 weights into the LMRS format:

python export.py --files [ordered .safetensor files] --config [model config.json] --save-path [name and path to save] --type [model type (GEMMA/LLAMA/PHI)]
To export the quantized version use the --quantize and --quantize-type flags. The int8 quantized model size should be 4X smaller (from ~9.8G to ~2.5G, depending on the group size). For multimodal models include the --vision-config argument.

Use the tokenizer.py script to convert the tokenizer model into the LMRS tokenizer format:

python tokenizer.py --model-id [huggingface model_id] --tokenizer-type [type of the tokenizer (GEMMA/LLAMA/PHI)]

Build

Compile the rust code with cargo (make sure to pass the target-cpu flag):

To run the backend for the WebUI, first compile:

RUSTFLAGS="-C target-cpu=native" cargo build --release --features multimodal --features backend --bin backend

You can change the ip and port with --ip and --port. Other flags such as temperature, etc. are also available. For multimodal compatibility use the --multimodal flag. You can now connect via the web interface.

then run:

./target/release/backend --model [model weights file]
