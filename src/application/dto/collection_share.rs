use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::collection_share::{CollectionShare, ShareRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionShareResponse {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub role: ShareRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CollectionShare> for CollectionShareResponse {
    fn from(s: CollectionShare) -> Self {
        Self {
            id: s.id,
            collection_id: s.collection_id,
            user_id: s.user_id,
            group_id: s.group_id,
            role: s.role,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShareRequest {
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub role: ShareRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShareRequest {
    pub role: ShareRole,
}
