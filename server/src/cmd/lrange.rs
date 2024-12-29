use crate::cmd::CommandError;
use crate::resp::types::RespType;
use crate::storage::db::DB;

#[derive(Debug, Clone)]
pub struct LRange {
    key: String,
    start: i64,
    stop: i64,
}

impl LRange {
    /// Creates a new LRange instance from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<LRange, CommandError> {
        if args.len() != 3 {
            return Err(CommandError::InvalidFormat);
        }

        let key = match &args[0] {
            RespType::BulkString(s) => s.clone(),
            _ => return Err(CommandError::InvalidFormat),
        };

        let start = Self::parse_idx(&args[1])?;

        let stop = Self::parse_idx(&args[2])?;

        Ok(LRange { key, start, stop })
    }

    /// Execute the LRange command.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.lrange(self.key.clone(), self.start, self.stop) {
            Ok(elems) => {
                let sub_list = elems
                    .iter()
                    .cloned()
                    .map(|e| RespType::BulkString(e))
                    .collect();
                RespType::Array(sub_list)
            }
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }

    fn parse_idx(v: &RespType) -> Result<i64, CommandError> {
        match v {
            RespType::BulkString(v) => {
                let start_idx = v.parse::<i64>();
                match start_idx {
                    Ok(i) => Ok(i),
                    Err(_) => Err(CommandError::Other(String::from(
                        "index should be an integer",
                    ))),
                }
            }
            _ => Err(CommandError::Other(String::from(
                "Invalid argument. Value must be an integer in bulk string format",
            ))),
        }
    }
}
