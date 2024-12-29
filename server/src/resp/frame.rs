use core::fmt;
use crate::resp::types::RespType;
use bytes::{Buf, BytesMut};
use std::fmt::{Debug, Formatter};
use std::io::Error;
use tokio_util::codec::{Decoder, Encoder};

use super::RespError;

const SIZE_OF_RESP_LEN: usize = 1 + std::mem::size_of::<usize>() + 2;

/// This codec handles commands, which are always represented as an array of bulk strings
/// in the RESP protocol.
pub struct RespCommandFrame {
    /// Builder for appending the bulk strings in the command array.
    cmd_builder: Option<CommandBuilder>,
}

impl RespCommandFrame {
    /// Create a new `RespCommandFrame`.
    pub fn new() -> Self {
        RespCommandFrame { cmd_builder: None }
    }
}

impl Encoder<RespType> for RespCommandFrame {
    type Error = std::io::Error;

    /// Encodes a `RespType` into bytes and writes them to the output buffer.
    fn encode(&mut self, item: RespType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend(item.to_bytes());
        Ok(())
    }
}

impl Decoder for RespCommandFrame {
    type Item = Vec<RespType>;
    type Error = std::io::Error;

    /// Decodes bytes from the input stream into a `Vec<RespType>` representing a command.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            src.advance(src.len());
            return Ok(None);
        }

        if self.cmd_builder.is_none() {
            let n = src.len().min(SIZE_OF_RESP_LEN);
            let (cmd_len, bytes_consumed) =
                match RespType::parse_array_len(BytesMut::from(&src[..n])) {
                    Ok((len, consumed)) => (len, consumed),
                    Err(e) => {
                        return Err(Error::new(
                            std::io::ErrorKind::InvalidData,
                            FrameError::from(e),
                        ));
                    },
                };

            self.cmd_builder = Some(CommandBuilder::new(cmd_len));
            src.advance(bytes_consumed);
        }

        // Read all bytes in the buffer
        while src.len() != 0 {
            let n = src.len().min(SIZE_OF_RESP_LEN);
            let (bulkstr_len, _) = match RespType::parse_bulk_string_len(
                BytesMut::from(&src[..n]),
            ) {
                Ok((len, consumed)) => (len, consumed),
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        FrameError::from(e),
                    ));
                },
            };


            let n = src.len().min(SIZE_OF_RESP_LEN + bulkstr_len + 2);
            let (bulkstr, bytes_consumed) = match RespType::parse_bulk_string(
                BytesMut::from(&src[..n]),
            ) {
                Ok((resp_type, bytes_read)) => (resp_type, bytes_read),
                Err(e) => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        FrameError::from(e),
                    ));
                }
            };

            self.cmd_builder.as_mut().unwrap().add_part(bulkstr);
            src.advance(bytes_consumed);

            let cmd_builder = self.cmd_builder.as_ref().unwrap();
            if cmd_builder.all_parts_received() {
                let cmd = cmd_builder.build();
                self.cmd_builder = None;
                return Ok(Some(cmd));
            }
        }

        Ok(None)
    }
}

/// This struct is used to accumulate the parts of a command,
/// which are typically represented as an array of bulk strings in the RESP protocol.
struct CommandBuilder {
    parts: Vec<RespType>,
    num_parts: usize,
    parts_parsed: usize,
}

impl CommandBuilder {
    /// Create a new `CommandBuilder` with the given number of parts.
    pub fn new(num_parts: usize) -> Self {
        CommandBuilder {
            parts: vec![],
            num_parts,
            parts_parsed: 0,
        }
    }

    /// Add a part to the command.
    pub fn add_part(&mut self, part: RespType) {
        self.parts.push(part);
        self.parts_parsed += 1;
    }

    /// Checks if all parts have been received.
    pub fn all_parts_received(&self) -> bool {
        self.parts_parsed == self.num_parts
    }

    /// Builds and returns the complete command as a vector of RESP values.
    pub fn build(&self) -> Vec<RespType> {
        self.parts.clone()
    }
}

/// Represents error that can occur during RESP command frame parsing.
#[derive(Debug)]
pub struct FrameError {
    err: RespError,
}

impl FrameError {
    pub fn from(err: RespError) -> Self {
        FrameError { err }
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.err, f)
    }
}

impl std::error::Error for FrameError {}

impl Into<Error> for FrameError {
    fn into(self) -> Error {
        Error::new(std::io::ErrorKind::InvalidData, self.to_string())
    }
}
