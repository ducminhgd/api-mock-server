use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::Row;
use uuid::Uuid;

use crate::application::repositories::collection_share::CollectionShareRepository;
use crate::domain::collection_share::{CollectionShare, ShareRole};
use crate::domain::errors::DomainError;

pub struct SqlxCollectionShareRepository {
    pool: AnyPool,
}

impl SqlxCollectionShareRepository {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|d| d.and_utc())
        })
        .map_err(|e| DomainError::Internal(format!("invalid timestamp '{s}': {e}")))
}

fn row_to_share(row: &sqlx::any::AnyRow) -> Result<CollectionShare, DomainError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let collection_id_str: String = row
        .try_get("collection_id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let collection_id =
        Uuid::parse_str(&collection_id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let user_id_str: Option<String> = row
        .try_get("user_id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let user_id = user_id_str
        .map(|s| Uuid::parse_str(&s).map_err(|e| DomainError::Internal(e.to_string())))
        .transpose()?;
    let group_id_str: Option<String> = row
        .try_get("group_id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let group_id = group_id_str
        .map(|s| Uuid::parse_str(&s).map_err(|e| DomainError::Internal(e.to_string())))
        .transpose()?;
    let role_str: String = row
        .try_get("role")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let role = role_str
        .parse::<ShareRole>()
        .map_err(DomainError::Internal)?;
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(CollectionShare {
        id,
        collection_id,
        user_id,
        group_id,
        role,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

#[async_trait]
impl CollectionShareRepository for SqlxCollectionShareRepository {
    async fn find_by_collection(
        &self,
        collection_id: Uuid,
    ) -> Result<Vec<CollectionShare>, DomainError> {
        let cid = collection_id.to_string();
        let rows = sqlx::query(
            "SELECT id, collection_id, user_id, group_id, role, created_at, updated_at \
             FROM collection_shares WHERE collection_id = ?",
        )
        .bind(&cid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.iter().map(row_to_share).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<CollectionShare, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, collection_id, user_id, group_id, role, created_at, updated_at \
             FROM collection_shares WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::CollectionShareNotFound(id))?;

        row_to_share(&row)
    }

    async fn find_existing(
        &self,
        collection_id: Uuid,
        user_id: Option<Uuid>,
        group_id: Option<Uuid>,
    ) -> Result<Option<CollectionShare>, DomainError> {
        let cid = collection_id.to_string();
        let uid = user_id.map(|u| u.to_string());
        let gid = group_id.map(|g| g.to_string());
        let row = sqlx::query(
            "SELECT id, collection_id, user_id, group_id, role, created_at, updated_at \
             FROM collection_shares \
             WHERE collection_id = ? \
               AND (user_id IS ? OR (user_id IS NULL AND ? IS NULL)) \
               AND (group_id IS ? OR (group_id IS NULL AND ? IS NULL))",
        )
        .bind(&cid)
        .bind(&uid)
        .bind(&uid)
        .bind(&gid)
        .bind(&gid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.as_ref().map(row_to_share).transpose()
    }

    async fn save(&self, share: &CollectionShare) -> Result<(), DomainError> {
        let id = share.id.to_string();
        let collection_id = share.collection_id.to_string();
        let user_id = share.user_id.map(|u| u.to_string());
        let group_id = share.group_id.map(|g| g.to_string());
        let role = share.role.to_string();
        let created_at = share.created_at.to_rfc3339();
        let updated_at = share.updated_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO collection_shares \
               (id, collection_id, user_id, group_id, role, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET role = excluded.role, updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(&collection_id)
        .bind(&user_id)
        .bind(&group_id)
        .bind(&role)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM collection_shares WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError> {
        let cid = collection_id.to_string();
        sqlx::query("DELETE FROM collection_shares WHERE collection_id = ?")
            .bind(&cid)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
