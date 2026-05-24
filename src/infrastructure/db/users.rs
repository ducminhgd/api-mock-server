use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::Row;
use uuid::Uuid;

fn parse_dt(s: &str) -> Result<DateTime<Utc>, crate::domain::errors::DomainError> {
    use crate::domain::errors::DomainError;
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|d| d.and_utc())
        })
        .map_err(|e| DomainError::Internal(format!("invalid timestamp '{s}': {e}")))
}

use crate::application::dto::pagination::PageParams;
use crate::application::dto::user::UserFilter;
use crate::application::repositories::user::UserRepository;
use crate::domain::errors::DomainError;
use crate::domain::user::{User, UserRole, UserStatus};

pub struct SqlxUserRepository {
    pool: AnyPool,
}

impl SqlxUserRepository {
    pub fn new(pool: AnyPool) -> Self {
        Self { pool }
    }
}

fn row_to_user(row: &sqlx::any::AnyRow) -> Result<User, DomainError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
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
        .parse::<UserRole>()
        .map_err(DomainError::Internal)?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let status = status_str
        .parse::<UserStatus>()
        .map_err(DomainError::Internal)?;
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    Ok(User {
        id,
        username: row
            .try_get("username")
            .map_err(|e| DomainError::Internal(e.to_string()))?,
        password_hash: row
            .try_get("password_hash")
            .map_err(|e| DomainError::Internal(e.to_string()))?,
        group_id,
        role,
        status,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_all(
        &self,
        filter: &UserFilter,
        page: &PageParams,
    ) -> Result<(Vec<User>, u64), DomainError> {
        let search = filter.search.as_deref().map(|s| format!("%{s}%"));
        let group_id = filter.group_id.map(|u| u.to_string());
        let status = filter.status.as_ref().map(|s| s.to_string());
        let limit = page.clamped_limit() as i64;
        let offset = page.offset() as i64;

        let rows = sqlx::query(
            "SELECT id, username, password_hash, group_id, role, status, \
                    CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at \
             FROM users \
             WHERE (? IS NULL OR username LIKE ?) \
               AND (? IS NULL OR group_id = ?) \
               AND (? IS NULL OR status = ?) \
             ORDER BY username ASC LIMIT ? OFFSET ?",
        )
        .bind(&search)
        .bind(&search)
        .bind(&group_id)
        .bind(&group_id)
        .bind(&status)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users \
             WHERE (? IS NULL OR username LIKE ?) \
               AND (? IS NULL OR group_id = ?) \
               AND (? IS NULL OR status = ?)",
        )
        .bind(&search)
        .bind(&search)
        .bind(&group_id)
        .bind(&group_id)
        .bind(&status)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let users = rows
            .iter()
            .map(row_to_user)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((users, total as u64))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, username, password_hash, group_id, role, status, \
                    CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at \
             FROM users WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::UserNotFound(id))?;

        row_to_user(&row)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, group_id, role, status, \
                    CAST(created_at AS TEXT) as created_at, CAST(updated_at AS TEXT) as updated_at \
             FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        row.as_ref().map(row_to_user).transpose()
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let id = user.id.to_string();
        let group_id = user.group_id.map(|u| u.to_string());
        let role = user.role.to_string();
        let status = user.status.to_string();
        let created_at = user.created_at.to_rfc3339();
        let updated_at = user.updated_at.to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, group_id, role, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               username = excluded.username, password_hash = excluded.password_hash, \
               group_id = excluded.group_id, role = excluded.role, \
               status = excluded.status, updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&group_id)
        .bind(&role)
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
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
