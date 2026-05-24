use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::Row;
use uuid::Uuid;

use crate::application::dto::collection::CollectionFilter;
use crate::application::dto::pagination::PageParams;
use crate::application::repositories::collection::CollectionRepository;
use crate::domain::collection::{Collection, CollectionStatus, CollectionVisibility};
use crate::domain::errors::DomainError;

pub struct SqlxCollectionRepository {
    pool: AnyPool,
}

impl SqlxCollectionRepository {
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

fn row_to_collection(row: &sqlx::any::AnyRow) -> Result<Collection, DomainError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let owner_id_str: String = row
        .try_get("owner_id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let owner_id =
        Uuid::parse_str(&owner_id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let status = status_str
        .parse::<CollectionStatus>()
        .map_err(DomainError::Internal)?;
    let visibility_str: String = row
        .try_get("visibility")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let visibility = visibility_str
        .parse::<CollectionVisibility>()
        .map_err(DomainError::Internal)?;
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let code: Option<String> = row.try_get("code").ok();
    Ok(Collection {
        id,
        name: row
            .try_get("name")
            .map_err(|e| DomainError::Internal(e.to_string()))?,
        code: code.unwrap_or_else(|| id_str.clone()),
        description: row
            .try_get("description")
            .map_err(|e| DomainError::Internal(e.to_string()))?,
        owner_id,
        status,
        visibility,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

#[async_trait]
impl CollectionRepository for SqlxCollectionRepository {
    async fn find_all(
        &self,
        caller_id: Uuid,
        caller_group_id: Option<Uuid>,
        filter: &CollectionFilter,
        page: &PageParams,
    ) -> Result<(Vec<Collection>, u64), DomainError> {
        let caller_id_str = caller_id.to_string();
        let caller_group_id_str = caller_group_id.map(|g| g.to_string());
        let search = filter.search.as_deref().map(|s| format!("%{s}%"));
        let status = filter.status.as_ref().map(|s| s.to_string());
        let visibility = filter.visibility.as_ref().map(|v| v.to_string());
        let limit = page.clamped_limit() as i64;
        let offset = page.offset() as i64;

        let rows = sqlx::query(
            "SELECT DISTINCT c.id, c.name, c.code, c.description, c.owner_id, c.status, c.visibility, \
                    CAST(c.created_at AS TEXT) as created_at, CAST(c.updated_at AS TEXT) as updated_at \
             FROM collections c \
             LEFT JOIN collection_shares cs ON cs.collection_id = c.id \
             WHERE (c.owner_id = ? OR cs.user_id = ? OR (? IS NOT NULL AND cs.group_id = ?)) \
               AND (? IS NULL OR c.name LIKE ?) \
               AND (? IS NULL OR c.status = ?) \
               AND (? IS NULL OR c.visibility = ?) \
             ORDER BY c.name ASC LIMIT ? OFFSET ?",
        )
        .bind(&caller_id_str)
        .bind(&caller_id_str)
        .bind(&caller_group_id_str)
        .bind(&caller_group_id_str)
        .bind(&search)
        .bind(&search)
        .bind(&status)
        .bind(&status)
        .bind(&visibility)
        .bind(&visibility)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT c.id) \
             FROM collections c \
             LEFT JOIN collection_shares cs ON cs.collection_id = c.id \
             WHERE (c.owner_id = ? OR cs.user_id = ? OR (? IS NOT NULL AND cs.group_id = ?)) \
               AND (? IS NULL OR c.name LIKE ?) \
               AND (? IS NULL OR c.status = ?) \
               AND (? IS NULL OR c.visibility = ?)",
        )
        .bind(&caller_id_str)
        .bind(&caller_id_str)
        .bind(&caller_group_id_str)
        .bind(&caller_group_id_str)
        .bind(&search)
        .bind(&search)
        .bind(&status)
        .bind(&status)
        .bind(&visibility)
        .bind(&visibility)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let collections = rows
            .iter()
            .map(row_to_collection)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((collections, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Collection, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, name, code, description, owner_id, status, visibility, \
                    CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at \
             FROM collections WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::CollectionNotFound(id))?;

        row_to_collection(&row)
    }

    async fn save(&self, collection: &Collection) -> Result<(), DomainError> {
        let id = collection.id.to_string();
        let owner_id = collection.owner_id.to_string();
        let status = collection.status.to_string();
        let visibility = collection.visibility.to_string();
        let created_at = collection.created_at.to_rfc3339();
        let updated_at = collection.updated_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO collections \
               (id, name, code, description, owner_id, status, visibility, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, code = excluded.code, description = excluded.description, \
               owner_id = excluded.owner_id, status = excluded.status, \
               visibility = excluded.visibility, updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(&collection.name)
        .bind(&collection.code)
        .bind(&collection.description)
        .bind(&owner_id)
        .bind(&status)
        .bind(&visibility)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn find_by_code(&self, code: &str) -> Result<Collection, DomainError> {
        let row = sqlx::query(
            "SELECT id, name, code, description, owner_id, status, visibility, \
                    CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at \
             FROM collections WHERE code = ?",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or_else(|| DomainError::InvalidInput(format!("collection code not found: {code}")))?;
        row_to_collection(&row)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
