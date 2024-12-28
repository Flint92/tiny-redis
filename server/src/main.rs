// Include the server module
mod server;

mod handler;
mod resp;

use anyhow::Result;
use log::{error, info};
use std::process::exit;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    // This sets up logging based on the RUST_LOG environment variable.
    env_logger::init();

    // Define the address the server will listen on.
    let addr = format!("127.0.0.1:{}", 16379);

    let listener = match TcpListener::bind(&addr).await {
        // If the binding is successful, the listener is returned.
        Ok(listener) => {
            info!("Listening on: {}", addr);
            listener
        }
        Err(e) => {
            error!("Could not bind the TCP listener to {}. Err: {}", &addr, e);
            exit(0)
        }
    };

    // Create a new server instance with the listener.
    let mut server = server::Server::new(listener);

    // Run the server to start accepting and handling incoming connections.
    // This will run infinitely until the server is stopped.
    server.run().await?;

    Ok(())
}
