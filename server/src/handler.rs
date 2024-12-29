use crate::cmd::tx::Transaction;
use crate::cmd::Command;
use crate::resp::frame::RespCommandFrame;
use crate::resp::types::RespType;
use crate::storage::db::DB;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

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
    pub async fn handle(&mut self, db: &DB) -> Result<()> {
        // commands are queued here if MULTI command was issued
        let mut multicommand = Transaction::new();

        while let Some(resp_cmd) = self.conn.next().await {
            match resp_cmd {
                Ok(cmd_frame) => {
                    // Read the command from the frame.
                    let resp_cmd = Command::from_resp_command_frame(cmd_frame);

                    let response = match resp_cmd {
                        Ok(cmd) => match cmd { 
                            Command::Multi => {
                                let init_multi_command = &mut multicommand.init();
                                match init_multi_command {
                                    Ok(_) => cmd.execute(db),
                                    Err(e) => RespType::SimpleError(format!("{}", e)),
                                }
                            },
                            Command::Exec => {
                                if multicommand.is_active() {
                                    multicommand.execute(db).await
                                } else {
                                    RespType::SimpleError(String::from("EXEC without MULTI"))
                                }
                            },
                            Command::Discard => {
                                if multicommand.is_active() {
                                    multicommand.discard();
                                    RespType::SimpleString(String::from("OK"))
                                } else {
                                    RespType::SimpleError(String::from("DISCARD without MULTI"))
                                }
                            },
                            _ => {
                                if multicommand.is_active() {
                                    multicommand.add_command(cmd);
                                    RespType::SimpleString(String::from("QUEUED"))
                                } else {
                                    cmd.execute(db)
                                }
                            }
                        },
                        Err(e) => {
                            if multicommand.is_active() { 
                                multicommand.discard();
                            }
                            RespType::SimpleError(format!("{}", e))
                        },
                    };

                    // Write the RESP response into the TCP stream.
                    if let Err(e) = self.conn.send(response).await {
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
