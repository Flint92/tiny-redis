use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::storage::DBError;

/// The Storage struct is designed to act as a wrapper around the core database,
/// allowing it to be shared across multiple connections. The database is encapsulated within an Arc,
/// to enable concurrent access.
#[derive(Debug, Clone)]
pub struct Storage {
    db: Arc<DB>,
}

/// The `DB` struct is the component that houses the actual data,
/// which is stored in a RwLock wrapped around a hashmap.
/// This ensures that the data can be accessed concurrently
#[derive(Debug)]
pub struct DB {
    data: RwLock<HashMap<String, Entry>>
}

/// The `Entry` struct represents the value associated with a particular key.
#[derive(Debug, Clone)]
pub struct Entry {
    value: Value,
}

/// The `Value` enum allows for storing various types of data associated with a key.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
}

impl Storage {
    /// Creates a new instance of the Storage struct.
    pub fn new() -> Self {
        Storage {
            db: Arc::new(DB::new())
        }
    }

    /// Returns a clone of the Arc wrapped DB instance.
    pub fn db(&self) -> Arc<DB> {
        self.db.clone()
    }
}

impl DB {
    /// Creates a new instance of the DB struct.
    pub fn new() -> Self {
        DB {
            data: RwLock::new(HashMap::new())
        }
    }

    /// Get the string value associated with a key.
    pub fn get(&self, key: &str) -> Result<Option<String>, DBError> {
        let data = match self.data.read() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.get(key) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        if let Value::String(s) = &entry.value {
            return Ok(Some(s.to_string()))
        }

        Err(DBError::WrongType)
    }

    /// Set the value associated with a key.
    pub fn set(&self, key: String, value: Value) -> Result<(), DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        data.insert(key, Entry { value });
        Ok(())
    }

}
