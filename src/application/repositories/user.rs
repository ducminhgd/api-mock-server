use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::pagination::PageParams;
use crate::application::dto::user::UserFilter;
use crate::domain::errors::DomainError;
use crate::domain::user::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(
        &self,
        filter: &UserFilter,
        page: &PageParams,
    ) -> Result<(Vec<User>, u64), DomainError>;

    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;

    async fn save(&self, user: &User) -> Result<(), DomainError>;

    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
