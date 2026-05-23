use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::Row;
use uuid::Uuid;

use crate::application::dto::group::GroupFilter;
use crate::application::dto::pagination::PageParams;
use crate::application::repositories::group::GroupRepository;
use crate::domain::errors::DomainError;
use crate::domain::group::{Group, GroupStatus};

pub struct SqlxGroupRepository {
    pool: AnyPool,
}

impl SqlxGroupRepository {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            // SQLite stores as "YYYY-MM-DD HH:MM:SS" when inserted via CURRENT_TIMESTAMP
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|d| d.and_utc())
        })
        .map_err(|e| DomainError::Internal(format!("invalid timestamp '{s}': {e}")))
}

fn row_to_group(row: &sqlx::any::AnyRow) -> Result<Group, DomainError> {
    let id_str: String = row.try_get("id").map_err(|e| DomainError::Internal(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let status_str: String = row.try_get("status").map_err(|e| DomainError::Internal(e.to_string()))?;
    let status = status_str.parse::<GroupStatus>().map_err(DomainError::Internal)?;
    let created_at_str: String = row.try_get("created_at").map_err(|e| DomainError::Internal(e.to_string()))?;
    let updated_at_str: String = row.try_get("updated_at").map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(Group {
        id,
        name: row.try_get("name").map_err(|e| DomainError::Internal(e.to_string()))?,
        description: row.try_get("description").map_err(|e| DomainError::Internal(e.to_string()))?,
        status,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

#[async_trait]
impl GroupRepository for SqlxGroupRepository {
    async fn find_all(
        &self,
        filter: &GroupFilter,
        page: &PageParams,
    ) -> Result<(Vec<Group>, u64), DomainError> {
        let search = filter.search.as_deref().map(|s| format!("%{s}%"));
        let status = filter.status.as_ref().map(|s| s.to_string());
        let limit = page.clamped_limit() as i64;
        let offset = page.offset() as i64;

        let rows = sqlx::query(
            "SELECT id, name, description, status, created_at, updated_at FROM groups \
             WHERE (? IS NULL OR name LIKE ?) AND (? IS NULL OR status = ?) \
             ORDER BY name ASC LIMIT ? OFFSET ?",
        )
        .bind(&search)
        .bind(&search)
        .bind(&status)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM groups WHERE (? IS NULL OR name LIKE ?) AND (? IS NULL OR status = ?)",
        )
        .bind(&search)
        .bind(&search)
        .bind(&status)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let groups = rows.iter().map(row_to_group).collect::<Result<Vec<_>, _>>()?;
        Ok((groups, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Group, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, name, description, status, created_at, updated_at FROM groups WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::GroupNotFound(id))?;

        row_to_group(&row)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Group>, DomainError> {
        let row = sqlx::query(
            "SELECT id, name, description, status, created_at, updated_at FROM groups WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.as_ref().map(row_to_group).transpose()
    }

    async fn save(&self, group: &Group) -> Result<(), DomainError> {
        let id = group.id.to_string();
        let status = group.status.to_string();
        let created_at = group.created_at.to_rfc3339();
        let updated_at = group.updated_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO groups (id, name, description, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, description = excluded.description, \
               status = excluded.status, updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(&group.name)
        .bind(&group.description)
        .bind(&status)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
