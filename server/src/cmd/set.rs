use crate::cmd::CommandError;
use crate::resp::types::RespType;
use crate::storage::db::{Value, DB};

/// Represents the set command.
#[derive(Debug)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {

    /// Creates a new instance of the set command from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<Self, CommandError> {
        if args.len() != 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'SET' command",
            )));
        }

        // parse key
        let key = &args[0];
        let key = match key {
            RespType::BulkString(key) => key,
            _ => return Err(CommandError::Other(String::from(
                "Invalid argument.Key must be a bulk string",
            ))),
        };

        // parse value
        let value = &args[1];
        let value = match value {
            RespType::BulkString(value) => value,
            _ => return Err(CommandError::Other(String::from(
                "Invalid argument.Value must be a bulk string",
            ))),
        };

        Ok(Self {
            key: key.to_string(),
            value: value.to_string(),
        })
    }

    /// Execute the set command.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.set(self.key.clone(), Value::String(self.value.clone())) {
            Ok(_) => RespType::BulkString("OK".to_string()),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
