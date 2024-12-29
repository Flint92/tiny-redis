use crate::cmd::CommandError;
use crate::resp::types::RespType;

/// Represents the PING command.
#[derive(Debug)]
pub struct Ping {
    msg: Option<String>
}

impl Ping {
    /// Creates a new PING instance from the given args.
    pub fn with_args(args: Vec<RespType>) -> Result<Ping, CommandError> {
        if args.len() == 0 {
            Ok(Ping { msg: None })
        } else if args.len() == 1 {
            match &args[0] {
                RespType::BulkString(s) => Ok(Ping { msg: Some(s.clone()) }),
                _ => Err(CommandError::Other(String::from("Invalid message"))),
            }
        } else {
            Err(CommandError::InvalidFormat)
        }
    }

    /// Executes the PING command.
    pub fn apply(&self) -> RespType {
        match &self.msg {
            Some(msg) => RespType::BulkString(msg.to_string()),
            None => RespType::SimpleString(String::from("PONG")),
        }
    }

}