use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::collection::CollectionFilter;
use crate::application::dto::pagination::PageParams;
use crate::domain::collection::Collection;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait CollectionRepository: Send + Sync {
    /// Returns collections visible to the caller — those owned by or shared with caller/caller's group.
    async fn find_all(
        &self,
        caller_id: Uuid,
        caller_group_id: Option<Uuid>,
        filter: &CollectionFilter,
        page: &PageParams,
    ) -> Result<(Vec<Collection>, u64), DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Collection, DomainError>;

    async fn save(&self, collection: &Collection) -> Result<(), DomainError>;

    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
