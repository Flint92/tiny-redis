use std::sync::Arc;
use crate::resp::types::RespType;
use anyhow::{Error, Result};
use bytes::BytesMut;
use log::error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use crate::handler::FrameHandler;
use crate::resp::frame::RespCommandFrame;
use crate::storage::db::Storage;

/// The server struct holds the tokio TcpListener which listens for
/// incoming TCP connections.
#[derive(Debug)]
pub struct Server {
    /// The TCP listener for accepting incoming connections.
    listener: TcpListener,
    /// Contains the shared storage.
    storage: Storage,
}

impl Server {
    /// Create a new server instance with the given TcpListener.
    pub fn new(listener: TcpListener, storage: Storage) -> Self {
        Self { listener, storage }
    }

    /// Runs the server in an infinite loop, continuously accepting and handling
    /// incoming connections.
    pub async fn run(&mut self) -> Result<()> {
        let db = self.storage.db();

        loop {
            // accept a new TCP connection
            // If successful the corresponding TcpStream is restored
            // in the `sock` variable, else a panic will occur.
            let sock = match self.accept_conn().await {
                Ok(sock) => sock,
                Err(e) => {
                    error!("Failed to accept incoming connection: {}", e);
                    panic!("Error accepting connection");
                }
            };

            let db = Arc::clone(&db);

            // Spawn a new asynchronous task to handle the incoming connection.
            // This allows the server to handle multiple connections concurrently.
            tokio::spawn(async move {
                // Use RespCommandFrame codec to read incoming TCP messages as Redis command frames,
                // and to write RespType values into outgoing TCP messages.
                let resp_command_frame= Framed::with_capacity(sock, RespCommandFrame::new(), 8 * 1024);

                // Create a new FrameHandler instance.
                let mut handler = FrameHandler::new(resp_command_frame);

                // Echo the RESP message back to the client.
                if let Err(e) = handler.handle(db.as_ref()).await {
                    // Log the error and panic if there is an issue writing the response.
                    error!("{}", e);
                    panic!("Error writing response")
                }

                // The connection is closed automatically when `sock` goes out the scope.
            });
        }
    }

    /// Accept a new incoming TCP connection and return the corresponding TcpStream.
    async fn accept_conn(&mut self) -> Result<TcpStream> {
        // Wait for an incoming connection.
        // The `accept()` method returns a tuple containing the TcpStream and the remote address,
        // but we are not interested in the remote address, so we use the `_` placeholder.
        match self.listener.accept().await {
            Ok((sock, _)) => Ok(sock),
            Err(e) => Err(Error::from(e)),
        }
    }
}
