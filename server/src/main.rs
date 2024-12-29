extern crate core;

// Include the server module
mod server;

mod handler;
mod resp;
mod cmd;

use anyhow::Result;
use log::{error, info};
use std::process::exit;
use tokio::net::TcpListener;
use clap::Parser;

const DEFAULT_PORT: u16 = 16379;

#[derive(Debug, Parser)]
#[command(
    name = "tiny-redis-server",
    version,
    author,
    about = "A RESP based in-memory cache"
)]
struct Cli {
    /// Port to be bound to tiny redis server
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    // This sets up logging based on the RUST_LOG environment variable.
    env_logger::init();

    // Get port from --port CLI parameter. Defaults to 16379
    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);

    // Define the address the server will listen on.
    let addr = format!("127.0.0.1:{}", port);

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
