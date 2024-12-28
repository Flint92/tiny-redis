use crate::resp::frame::RespCommandFrame;
use crate::resp::types::RespType;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, FramedWrite};

/// FrameHandler is a struct that holds the connection to the client and handles RESP command
pub struct FrameHandler {
    /// The framed connection using `RespCommandFrame` as the codec.
    conn: Framed<TcpStream, RespCommandFrame>,
}

impl FrameHandler {
    /// Create a new `FrameHandler` with the given connection.
    pub fn new(conn: Framed<TcpStream, RespCommandFrame>) -> Self {
        FrameHandler { conn }
    }

    /// Handle the incoming connection by reading the frames and processing the commands.
    pub async fn handle(&mut self) -> Result<()> {
        while let Some(resp_cmd) = self.conn.next().await {
            match resp_cmd {
                Ok(cmd_frame) => {
                    // Write the RESP response into the TCP stream.
                    if let Err(e) = self.conn.send(RespType::Array(cmd_frame)).await {
                        error!("Error sending response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading the request: {}", e);
                    break;
                }
            };

            // flush the buffer into the TCP stream.
            self.conn.flush().await?;
        }

        Ok(())
    }
}
