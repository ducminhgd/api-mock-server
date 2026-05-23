use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Clone)]
pub enum DomainError {
    #[error("group not found: {0}")]
    GroupNotFound(Uuid),
    #[error("user not found: {0}")]
    UserNotFound(Uuid),
    #[error("username already taken: {0}")]
    UsernameTaken(String),
    #[error("group name already taken: {0}")]
    GroupNameTaken(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("forbidden")]
    Forbidden,
    #[error("internal error: {0}")]
    Internal(String),
}
