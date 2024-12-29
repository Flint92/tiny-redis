use crate::cmd::{parse_values, CommandError};
use crate::resp::types::RespType;
use crate::storage::db::DB;

/// Represents the RPUSH command
#[derive(Debug, Clone)]
pub struct RPush {
    key: String,
    values: Vec<String>,
}

impl RPush {
    /// Creates a new RPush instance from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<RPush, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'RPUSH' command",
            )));
        }

        // parse key
        let key = match &args[0] {
            RespType::BulkString(s) => s.to_string(),
            _ => return Err(CommandError::Other(String::from("Invalid key"))),
        };

        // parse values
        let values = parse_values(args)?;

        Ok(RPush { key, values })
    }

    /// Executes the RPUSH command.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.rpush(self.key.clone(), self.values.clone()) {
            Ok(len) => RespType::Integer(len as i64),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }

    pub fn build_command(&self) -> RespType {
        let mut args: Vec<RespType> = vec![
            RespType::BulkString(String::from("RPUSH")),
            RespType::BulkString(self.key.clone()),
        ];

        let arg_vals = self.values.clone();
        for arg in arg_vals.iter() {
            args.push(RespType::BulkString(arg.to_string()));
        }

        RespType::Array(args)
    }
}
