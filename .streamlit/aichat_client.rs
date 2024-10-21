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