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