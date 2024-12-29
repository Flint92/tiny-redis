use crate::cmd::CommandError;
use crate::resp::types::RespType;
use crate::storage::db::DB;

/// Represents the get command
#[derive(Debug)]
pub struct Get {
    /// The key to get the value for.
    key: String,
}

impl Get {

    /// Create a new instance of the get command from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<Self, CommandError> {
        if args.len() != 1 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'GET' command",
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

        Ok(Self {
            key: key.to_string(),
        })
    }

    pub fn apply(&self, db: &DB) -> RespType {
        match db.get(self.key.as_str()) {
            Ok(Some(value)) => RespType::BulkString(value.to_string()),
            Ok(None) => RespType::NullBulkString,
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }

}