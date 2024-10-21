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