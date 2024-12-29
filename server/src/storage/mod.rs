use core::fmt;

pub mod db;

/// Represents errors that can occur during DB operations.
#[derive(Debug)]
pub enum DBError {
    /// Represents an error where wrong data type is encountered against a key.
    /// For e.g. If you try to perform list related operation (such as lpush, rpush) on a key
    /// which stores a string value.
    WrongType,
    /// Represents any other error with a descriptive message.
    Other(String),
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::WrongType => {
                "WRONG TYPE Operation against a key holding the wrong kind of value".fmt(f)
            }
            DBError::Other(msg) => msg.as_str().fmt(f),
        }
    }
}