//! Error types for persistence operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Duplicate entity: {0}")]
    Duplicate(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Mutation error: {0}")]
    MutationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Internal lock poisoned - a thread panicked while holding a lock")]
    LockPoisoned,
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;
