use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::endpoint::EndpointFilter;
use crate::application::dto::pagination::PageParams;
use crate::domain::endpoint::Endpoint;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait EndpointRepository: Send + Sync {
    async fn find_by_collection(
        &self,
        collection_id: Uuid,
        filter: &EndpointFilter,
        page: &PageParams,
    ) -> Result<(Vec<Endpoint>, u64), DomainError>;

    async fn find_all_by_collection(
        &self,
        collection_id: Uuid,
    ) -> Result<Vec<Endpoint>, DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Endpoint, DomainError>;

    async fn save(&self, endpoint: &Endpoint) -> Result<(), DomainError>;

    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError>;
}
