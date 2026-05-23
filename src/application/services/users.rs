use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::pagination::{PageParams, Paginated};
use crate::application::dto::user::{
    CreateUserRequest, ResetPasswordResponse, UpdateUserRequest, UserFilter, UserResponse,
};
use crate::application::repositories::user::UserRepository;
use crate::domain::errors::DomainError;
use crate::domain::user::{User, UserRole};

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, DomainError>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError>;
    fn generate_random(&self) -> String;
}

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { repo, hasher }
    }

    pub async fn list(
        &self,
        filter: UserFilter,
        page: PageParams,
    ) -> Result<Paginated<UserResponse>, DomainError> {
        let page = PageParams {
            limit: page.clamped_limit(),
            ..page
        };
        let (users, total) = self.repo.find_all(&filter, &page).await?;
        let data = users.into_iter().map(UserResponse::from).collect();
        Ok(Paginated::new(data, total, &page))
    }

    pub async fn get(&self, id: Uuid) -> Result<UserResponse, DomainError> {
        self.repo.find_by_id(id).await.map(UserResponse::from)
    }

    pub async fn create(&self, req: CreateUserRequest) -> Result<UserResponse, DomainError> {
        if self.repo.find_by_username(&req.username).await?.is_some() {
            return Err(DomainError::UsernameTaken(req.username));
        }
        let hash = self.hasher.hash(&req.password)?;
        let user = User::new(
            req.username,
            hash,
            req.group_id,
            req.role.unwrap_or(UserRole::Regular),
        );
        self.repo.save(&user).await?;
        Ok(UserResponse::from(user))
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateUserRequest,
    ) -> Result<UserResponse, DomainError> {
        let mut user = self.repo.find_by_id(id).await?;

        if let Some(ref username) = req.username {
            if let Some(existing) = self.repo.find_by_username(username).await? {
                if existing.id != id {
                    return Err(DomainError::UsernameTaken(username.clone()));
                }
            }
        }

        user.apply_update(req.username, req.group_id, req.status);
        self.repo.save(&user).await?;
        Ok(UserResponse::from(user))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.repo.find_by_id(id).await?;
        self.repo.delete(id).await
    }

    pub async fn reset_password(&self, id: Uuid) -> Result<ResetPasswordResponse, DomainError> {
        let mut user = self.repo.find_by_id(id).await?;
        let new_password = self.hasher.generate_random();
        let hash = self.hasher.hash(&new_password)?;
        user.set_password_hash(hash);
        self.repo.save(&user).await?;
        Ok(ResetPasswordResponse { password: new_password })
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::dto::pagination::PageParams;
    use crate::application::dto::user::{CreateUserRequest, UpdateUserRequest, UserFilter};
    use crate::application::services::fakes::{FakeHasher, FakeUserRepo};
    use crate::domain::errors::DomainError;
    use crate::domain::user::{User, UserRole, UserStatus};

    fn make_user(username: &str) -> User {
        User::new(username.into(), "hashed:pass".into(), None, UserRole::Regular)
    }

    fn svc(users: Vec<User>) -> UserService {
        UserService::new(Arc::new(FakeUserRepo::with(users)), Arc::new(FakeHasher))
    }

    fn empty() -> UserService {
        UserService::new(Arc::new(FakeUserRepo::empty()), Arc::new(FakeHasher))
    }

    #[tokio::test]
    async fn list_empty_store() {
        let result = empty()
            .list(UserFilter { search: None, group_id: None, status: None }, PageParams::default())
            .await
            .unwrap();
        assert!(result.data.is_empty());
        assert_eq!(result.meta.total, 0);
    }

    #[tokio::test]
    async fn list_filters_by_username_search() {
        let result = svc(vec![make_user("alice"), make_user("bob")])
            .list(
                UserFilter { search: Some("ali".into()), group_id: None, status: None },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].username, "alice");
    }

    #[tokio::test]
    async fn list_filters_by_group_id() {
        let gid = Uuid::new_v4();
        let mut u = make_user("alice");
        u.group_id = Some(gid);
        let result = svc(vec![u, make_user("bob")])
            .list(
                UserFilter { search: None, group_id: Some(gid), status: None },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].username, "alice");
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let mut inactive = make_user("alice");
        inactive.status = UserStatus::Inactive;
        let result = svc(vec![inactive, make_user("bob")])
            .list(
                UserFilter { search: None, group_id: None, status: Some(UserStatus::Active) },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].username, "bob");
    }

    #[tokio::test]
    async fn get_returns_user_by_id() {
        let u = make_user("carol");
        let id = u.id;
        let resp = svc(vec![u]).get(id).await.unwrap();
        assert_eq!(resp.id, id);
        assert_eq!(resp.username, "carol");
    }

    #[tokio::test]
    async fn get_returns_not_found_for_unknown_id() {
        let err = empty().get(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, DomainError::UserNotFound(_)));
    }

    #[tokio::test]
    async fn create_saves_new_user_with_regular_role_by_default() {
        let resp = empty()
            .create(CreateUserRequest { username: "dave".into(), password: "secret".into(), group_id: None, role: None })
            .await
            .unwrap();
        assert_eq!(resp.username, "dave");
        assert_eq!(resp.role, UserRole::Regular);
    }

    #[tokio::test]
    async fn create_uses_provided_role() {
        let resp = empty()
            .create(CreateUserRequest {
                username: "admin-user".into(),
                password: "pass".into(),
                group_id: None,
                role: Some(UserRole::Admin),
            })
            .await
            .unwrap();
        assert_eq!(resp.role, UserRole::Admin);
    }

    #[tokio::test]
    async fn create_rejects_duplicate_username() {
        let err = svc(vec![make_user("eve")])
            .create(CreateUserRequest { username: "eve".into(), password: "pass".into(), group_id: None, role: None })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::UsernameTaken(_)));
    }

    #[tokio::test]
    async fn update_changes_username() {
        let u = make_user("old");
        let id = u.id;
        let resp = svc(vec![u])
            .update(id, UpdateUserRequest { username: Some("new".into()), group_id: None, status: None })
            .await
            .unwrap();
        assert_eq!(resp.username, "new");
    }

    #[tokio::test]
    async fn update_rejects_duplicate_username_from_other_user() {
        let u1 = make_user("alice");
        let u2 = make_user("bob");
        let id2 = u2.id;
        let err = svc(vec![u1, u2])
            .update(id2, UpdateUserRequest { username: Some("alice".into()), group_id: None, status: None })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::UsernameTaken(_)));
    }

    #[tokio::test]
    async fn delete_removes_user() {
        let u = make_user("greta");
        let id = u.id;
        let service = svc(vec![u]);
        service.delete(id).await.unwrap();
        assert!(matches!(service.get(id).await.unwrap_err(), DomainError::UserNotFound(_)));
    }

    #[tokio::test]
    async fn delete_returns_not_found_for_unknown_id() {
        let err = empty().delete(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, DomainError::UserNotFound(_)));
    }

    #[tokio::test]
    async fn reset_password_returns_new_password_in_plaintext() {
        let u = make_user("hal");
        let id = u.id;
        let resp = svc(vec![u]).reset_password(id).await.unwrap();
        assert_eq!(resp.password, "random-password-abc123xyz!");
    }

    #[tokio::test]
    async fn reset_password_returns_not_found_for_unknown_id() {
        let err = empty().reset_password(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, DomainError::UserNotFound(_)));
    }
}
