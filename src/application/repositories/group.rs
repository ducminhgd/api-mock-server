use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::group::GroupFilter;
use crate::application::dto::pagination::PageParams;
use crate::domain::errors::DomainError;
use crate::domain::group::Group;

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn find_all(
        &self,
        filter: &GroupFilter,
        page: &PageParams,
    ) -> Result<(Vec<Group>, u64), DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Group, DomainError>;

    async fn find_by_name(&self, name: &str) -> Result<Option<Group>, DomainError>;

    async fn save(&self, group: &Group) -> Result<(), DomainError>;

    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
