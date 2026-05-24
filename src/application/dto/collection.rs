use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::collection::{Collection, CollectionStatus, CollectionVisibility};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub status: CollectionStatus,
    pub visibility: CollectionVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Collection> for CollectionResponse {
    fn from(c: Collection) -> Self {
        Self {
            id: c.id,
            name: c.name,
            code: c.code,
            description: c.description,
            owner_id: c.owner_id,
            status: c.status,
            visibility: c.visibility,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<CollectionVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollectionRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    /// `Some(None)` clears the description; `None` leaves it unchanged.
    pub description: Option<Option<String>>,
    pub status: Option<CollectionStatus>,
    pub visibility: Option<CollectionVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFilter {
    pub search: Option<String>,
    pub status: Option<CollectionStatus>,
    pub visibility: Option<CollectionVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: Uuid,
}
