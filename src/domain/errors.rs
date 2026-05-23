use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone)]
pub enum DomainError {
    #[error("group not found: {0}")]
    GroupNotFound(Uuid),
    #[error("user not found: {0}")]
    UserNotFound(Uuid),
    #[error("collection not found: {0}")]
    CollectionNotFound(Uuid),
    #[error("collection share not found: {0}")]
    CollectionShareNotFound(Uuid),
    #[error("username already taken: {0}")]
    UsernameTaken(String),
    #[error("group name already taken: {0}")]
    GroupNameTaken(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error: {0}")]
    Internal(String),
}
