use std::fmt::Formatter;

pub mod types;

/// This module contains the different errors that can occur while parsing RESP.
#[derive(Debug)]
pub enum RespError {
    InvalidBulkString(String),
    InvalidSimpleString(String),
    Other(String),
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::InvalidBulkString(s) => s.as_str().fmt(f),
            RespError::InvalidSimpleString(s) => s.as_str().fmt(f),
            RespError::Other(s) => s.as_str().fmt(f),
        }
    }
}

impl Into<std::io::Error> for RespError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, self.to_string())
    }
}