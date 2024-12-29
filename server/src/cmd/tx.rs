use crate::cmd::Command;
use crate::resp::types::RespType;
use crate::storage::db::DB;
use core::fmt;

/// Represents a transaction.
pub struct Transaction {
    /// The queue of commands to be executed.
    commands: Vec<Command>,
    /// Indicates whether a transaction is currently active.
    is_active: bool,
}

impl Transaction {
    /// Creates a new transaction.
    pub fn new() -> Transaction {
        Transaction {
            commands: Vec::new(),
            is_active: false,
        }
    }

    /// Initializes the transaction (MULTI command).
    pub fn init(&mut self) -> Result<(), TransactionError> {
        if self.is_active() {
            return Err(TransactionError::CannotNestMulti);
        }
        self.is_active = true;
        Ok(())
    }

    /// Checks if a transaction is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Adds a command to the transaction.
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// Executes the transaction.
    pub async fn execute(&mut self, db: &DB) -> RespType {
        let mut responses: Vec<RespType> = vec![];

        for cmd in self.commands.iter() {
            // execute the command
            let res = cmd.execute(db);

            responses.push(res);
        }

        // discard txn after executing all commands
        self.discard();

        RespType::Array(responses)
    }

    /// Discards the current transaction.
    pub fn discard(&mut self) {
        self.commands.clear();
        self.is_active = false;
    }
}

/// Represents errors that can occur during transaction operations.
#[derive(Debug)]
pub enum TransactionError {
    /// Indicates that a MULTI command cannot be nested within another active transaction.
    CannotNestMulti,
}

impl std::error::Error for TransactionError {}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::CannotNestMulti => "MULTI calls cannot be nested".fmt(f),
        }
    }
}
