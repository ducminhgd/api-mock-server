use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::collection_share::CollectionShare;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait CollectionShareRepository: Send + Sync {
    async fn find_by_collection(&self, collection_id: Uuid) -> Result<Vec<CollectionShare>, DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<CollectionShare, DomainError>;

    /// Returns an existing share for the given (collection, user) or (collection, group) pair.
    async fn find_existing(
        &self,
        collection_id: Uuid,
        user_id: Option<Uuid>,
        group_id: Option<Uuid>,
    ) -> Result<Option<CollectionShare>, DomainError>;

    async fn save(&self, share: &CollectionShare) -> Result<(), DomainError>;

    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError>;
}
