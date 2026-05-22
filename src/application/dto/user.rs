use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::user::{User, UserRole, UserStatus};

/// Returned to callers — never includes password_hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub group_id: Option<Uuid>,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            group_id: u.group_id,
            role: u.role,
            status: u.status,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub group_id: Option<Uuid>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    /// `Some(None)` removes group membership; `None` leaves it unchanged.
    pub group_id: Option<Option<Uuid>>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordResponse {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFilter {
    pub search: Option<String>,
    pub group_id: Option<Uuid>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}
