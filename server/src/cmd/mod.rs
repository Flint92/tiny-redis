use crate::cmd::ping::Ping;
use crate::resp::types::RespType;
use core::fmt;
use crate::cmd::get::Get;
use crate::cmd::lpush::LPush;
use crate::cmd::lrange::LRange;
use crate::cmd::rpush::RPush;
use crate::cmd::set::Set;
use crate::storage::db::DB;

pub mod ping;
mod set;
mod get;
mod lpush;
mod rpush;
mod lrange;

/// Represents a command.
#[derive(Debug)]
pub enum Command {
    /// The PING command.
    Ping(Ping),
    /// The SET command.
    Set(Set),
    /// The GET command.
    Get(Get),
    /// The LPUSH command.
    LPush(LPush),
    /// The RPUSH command.
    RPush(RPush),
    /// The LRANGE command.
    LRange(LRange),
}

impl Command {
    /// Attempts to create a Command from the given RESP command frame.
    pub fn from_resp_command_frame(frame: Vec<RespType>) -> Result<Command, CommandError> {
        if frame.len() == 0 {
            return Err(CommandError::InvalidFormat);
        }

        let (cmd_name, args) = frame.split_at(1);
        let cmd_name = match &cmd_name[0] {
            RespType::BulkString(s) => s.clone(),
            _ => return Err(CommandError::InvalidFormat),
        };

        let cmd = match cmd_name.to_lowercase().as_str() {
            "ping" => Command::Ping(Ping::with_args(args.to_vec())?),
            "set" => Command::Set(Set::with_args(args.to_vec())?),
            "get" => Command::Get(Get::with_args(args.to_vec())?),
            "lpush" => Command::LPush(LPush::with_args(args.to_vec())?),
            "rpush" => Command::RPush(RPush::with_args(args.to_vec())?),
            "lrange" => Command::LRange(LRange::with_args(args.to_vec())?),
            _ => {
                return Err(CommandError::UnknownCommand(ErrUnknownCommand {
                    cmd: cmd_name.to_string(),
                }))
            }
        };

        Ok(cmd)
    }

    /// Executes the command.
    pub fn execute(&self, db: &DB) -> RespType {
        match self {
            Command::Ping(ping) => ping.apply(),
            Command::Set(set) => set.apply(db),
            Command::Get(get) => get.apply(db),
            Command::LPush(lpush) => lpush.apply(db),
            Command::RPush(rpush) => rpush.apply(db),
            Command::LRange(lrange) => lrange.apply(db),
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    InvalidFormat,
    UnknownCommand(ErrUnknownCommand),
    Other(String),
}

#[derive(Debug)]
pub struct ErrUnknownCommand {
    pub cmd: String,
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::InvalidFormat => "Invalid command format".fmt(f),
            CommandError::UnknownCommand(e) => write!(f, "Unknown command: {}", e.cmd),
            CommandError::Other(msg) => msg.as_str().fmt(f),
        }
    }
}

fn parse_values(mut args: Vec<RespType>) -> Result<Vec<String>, CommandError> {
    let mut  values = Vec::new();

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
