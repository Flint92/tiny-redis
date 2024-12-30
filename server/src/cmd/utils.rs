use crate::cmd::CommandError;
use crate::resp::types::RespType;

pub fn parse_values(args: Vec<RespType>) -> Result<Vec<String>, CommandError> {
    let mut values = Vec::new();

    for arg in args.iter().skip(1) {
        match arg {
            RespType::BulkString(s) => values.push(s.to_string()),
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be a bulk string",
                )));
            }
        }
    }
    Ok(values)
}