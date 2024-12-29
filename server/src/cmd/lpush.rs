use crate::cmd::{parse_values, CommandError};
use crate::resp::types::RespType;
use crate::storage::db::DB;

/// Represents the LPUSH command
#[derive(Debug, Clone)]
pub struct LPush {
    key: String,
    values: Vec<String>,
}


impl LPush {

    /// Creates a new LPush instance from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<LPush, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'LPUSH' command",
            )));
        }

        // parse key
        let key = match &args[0] {
            RespType::BulkString(s) => s.to_string(),
            _ => return Err(CommandError::Other(String::from("Invalid key"))),
        };

        // parse values
        let values = parse_values(args)?;

        Ok(LPush { key, values })
    }

    /// Executes the LPUSH command.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.lpush(self.key.clone(), self.values.clone()) {
            Ok(len) => RespType::Integer(len as i64),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}