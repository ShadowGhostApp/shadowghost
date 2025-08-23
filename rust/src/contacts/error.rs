use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ContactError {
    InvalidContact(String),
    ContactNotFound(String),
    ContactExists(String),
    SerializationError(String),
    IoError(String),
}

impl fmt::Display for ContactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactError::InvalidContact(msg) => write!(f, "Invalid contact: {}", msg),
            ContactError::ContactNotFound(msg) => write!(f, "Contact not found: {}", msg),
            ContactError::ContactExists(msg) => write!(f, "Contact exists: {}", msg),
            ContactError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ContactError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for ContactError {}
