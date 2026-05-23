use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::Row;
use uuid::Uuid;

use crate::application::dto::endpoint::EndpointFilter;
use crate::application::dto::pagination::PageParams;
use crate::application::repositories::endpoint::EndpointRepository;
use crate::domain::endpoint::{Endpoint, EndpointStatus, HttpMethod};
use crate::domain::errors::DomainError;

pub struct SqlxEndpointRepository {
    pool: AnyPool,
}

impl SqlxEndpointRepository {
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

fn row_to_endpoint(row: &sqlx::any::AnyRow) -> Result<Endpoint, DomainError> {
    let id_str: String = row
        .try_get("id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let collection_id_str: String = row
        .try_get("collection_id")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let collection_id =
        Uuid::parse_str(&collection_id_str).map_err(|e| DomainError::Internal(e.to_string()))?;
    let name: String = row
        .try_get("name")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let method_str: String = row
        .try_get("method")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let method = method_str
        .parse::<HttpMethod>()
        .map_err(DomainError::Internal)?;
    let path: String = row
        .try_get("path")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let status_code: i64 = row
        .try_get("status_code")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let response_headers: Option<String> = row
        .try_get("response_headers")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let response_body: Option<String> = row
        .try_get("response_body")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let response_content_type: Option<String> = row
        .try_get("response_content_type")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let delay_ms: i64 = row
        .try_get("delay_ms")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let status_str: String = row
        .try_get("status")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let status = status_str
        .parse::<EndpointStatus>()
        .map_err(DomainError::Internal)?;
    let created_at_str: String = row
        .try_get("created_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;
    let updated_at_str: String = row
        .try_get("updated_at")
        .map_err(|e| DomainError::Internal(e.to_string()))?;

    Ok(Endpoint {
        id,
        collection_id,
        name,
        method,
        path,
        status_code: status_code as u16,
        response_headers,
        response_body,
        response_content_type,
        delay_ms: delay_ms as u32,
        status,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

#[async_trait]
impl EndpointRepository for SqlxEndpointRepository {
    async fn find_by_collection(
        &self,
        collection_id: Uuid,
        filter: &EndpointFilter,
        page: &PageParams,
    ) -> Result<(Vec<Endpoint>, u64), DomainError> {
        let cid = collection_id.to_string();
        let search = filter.search.as_deref().map(|s| format!("%{s}%"));
        let method = filter.method.as_ref().map(|m| m.to_string());
        let status = filter.status.as_ref().map(|s| s.to_string());
        let limit = page.clamped_limit() as i64;
        let offset = page.offset() as i64;

        let rows = sqlx::query(
            "SELECT id, collection_id, name, method, path, status_code, \
                    response_headers, response_body, response_content_type, \
                    delay_ms, status, created_at, updated_at \
             FROM endpoints \
             WHERE collection_id = ? \
               AND (? IS NULL OR (name LIKE ? OR path LIKE ?)) \
               AND (? IS NULL OR method = ?) \
               AND (? IS NULL OR status = ?) \
             ORDER BY name ASC \
             LIMIT ? OFFSET ?",
        )
        .bind(&cid)
        .bind(&search)
        .bind(&search)
        .bind(&search)
        .bind(&method)
        .bind(&method)
        .bind(&status)
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM endpoints \
             WHERE collection_id = ? \
               AND (? IS NULL OR (name LIKE ? OR path LIKE ?)) \
               AND (? IS NULL OR method = ?) \
               AND (? IS NULL OR status = ?)",
        )
        .bind(&cid)
        .bind(&search)
        .bind(&search)
        .bind(&search)
        .bind(&method)
        .bind(&method)
        .bind(&status)
        .bind(&status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        let endpoints = rows
            .iter()
            .map(row_to_endpoint)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((endpoints, total as u64))
    }

    async fn find_all_by_collection(
        &self,
        collection_id: Uuid,
    ) -> Result<Vec<Endpoint>, DomainError> {
        let cid = collection_id.to_string();
        let rows = sqlx::query(
            "SELECT id, collection_id, name, method, path, status_code, \
                    response_headers, response_body, response_content_type, \
                    delay_ms, status, created_at, updated_at \
             FROM endpoints WHERE collection_id = ?",
        )
        .bind(&cid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        rows.iter().map(row_to_endpoint).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Endpoint, DomainError> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, collection_id, name, method, path, status_code, \
                    response_headers, response_body, response_content_type, \
                    delay_ms, status, created_at, updated_at \
             FROM endpoints WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?
        .ok_or(DomainError::EndpointNotFound(id))?;

        row_to_endpoint(&row)
    }

    async fn save(&self, endpoint: &Endpoint) -> Result<(), DomainError> {
        let id = endpoint.id.to_string();
        let collection_id = endpoint.collection_id.to_string();
        let method = endpoint.method.to_string();
        let status_code = endpoint.status_code as i64;
        let delay_ms = endpoint.delay_ms as i64;
        let status = endpoint.status.to_string();
        let created_at = endpoint.created_at.to_rfc3339();
        let updated_at = endpoint.updated_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO endpoints \
               (id, collection_id, name, method, path, status_code, \
                response_headers, response_body, response_content_type, \
                delay_ms, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, method = excluded.method, path = excluded.path, \
               status_code = excluded.status_code, \
               response_headers = excluded.response_headers, \
               response_body = excluded.response_body, \
               response_content_type = excluded.response_content_type, \
               delay_ms = excluded.delay_ms, status = excluded.status, \
               updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(&collection_id)
        .bind(&endpoint.name)
        .bind(&method)
        .bind(&endpoint.path)
        .bind(status_code)
        .bind(&endpoint.response_headers)
        .bind(&endpoint.response_body)
        .bind(&endpoint.response_content_type)
        .bind(delay_ms)
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
        sqlx::query("DELETE FROM endpoints WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError> {
        let cid = collection_id.to_string();
        sqlx::query("DELETE FROM endpoints WHERE collection_id = ?")
            .bind(&cid)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}
