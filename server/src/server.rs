use log::error;
use std::io::Error;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::{TcpListener, TcpStream};
use crate::resp::types::RespType;

/// The server struct holds the tokio TcpListener which listens for
/// incoming TCP connections.
#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Create a new server instance with the given TcpListener.
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Runs the server in an infinite loop, continuously accepting and handling
    /// incoming connections.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // accept a new TCP connection
            // If successful the corresponding TcpStream is restored
            // in the `sock` variable, else a panic will occur.
            let mut sock = match self.accept_conn().await {
                Ok(sock) => sock,
                Err(e) => {
                    error!("Failed to accept incoming connection: {}", e);
                    panic!("Error accepting connection");
                }
            };

            // Spawn a new asynchronous task to handle the incoming connection.
            // This allows the server to handle multiple connections concurrently.
            tokio::spawn(async move {
                // Create a buffer to store the incoming data.
                let mut buf = BytesMut::with_capacity(512);
                if let Err(e) = sock.read_buf(&mut buf).await {
                    panic!("Failed to read from socket; err = {:?}", e);
                }

                // Try parsing the RESP data from the bytes in the buffer.
                let resp_data = match RespType::parse(buf) {
                    Ok((data, _)) => data,
                    Err(e) => RespType::SimpleError(format!("{}", e))
                };

                // Echo the RESP message back to the client.
                if let Err(e) = &mut sock.write_all(&resp_data.to_bytes()[..]).await {
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
