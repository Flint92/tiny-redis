use core::fmt;
use std::fmt::Formatter;

pub mod frame;
pub mod types;

/// This module contains the different errors that can occur while parsing RESP.
#[derive(Debug)]
pub enum RespError {
    InvalidBulkString(String),
    InvalidSimpleString(String),
    InvalidArray(String),
    Other(String),
}

impl fmt::Display for RespError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::InvalidBulkString(s) => s.as_str().fmt(f),
            RespError::InvalidSimpleString(s) => s.as_str().fmt(f),
            RespError::InvalidArray(s) => s.as_str().fmt(f),
            RespError::Other(s) => s.as_str().fmt(f),
        }
    }
}
