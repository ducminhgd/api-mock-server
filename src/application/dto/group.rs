use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::group::{Group, GroupStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: GroupStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Group> for GroupResponse {
    fn from(g: Group) -> Self {
        Self {
            id: g.id,
            name: g.name,
            description: g.description,
            status: g.status,
            created_at: g.created_at,
            updated_at: g.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    /// `Some(None)` clears the description; `None` leaves it unchanged.
    pub description: Option<Option<String>>,
    pub status: Option<GroupStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFilter {
    pub search: Option<String>,
    pub status: Option<GroupStatus>,
}
