use std::sync::Arc;

use crate::application::dto::user::{LoginRequest, LoginResponse, UserResponse};
use crate::application::repositories::user::UserRepository;
use crate::application::services::users::PasswordHasher;
use crate::domain::errors::DomainError;

pub trait TokenIssuer: Send + Sync {
    fn issue(&self, user_id: &str, role: &str) -> Result<String, DomainError>;
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    issuer: Arc<dyn TokenIssuer>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        issuer: Arc<dyn TokenIssuer>,
    ) -> Self {
        Self {
            user_repo,
            hasher,
            issuer,
        }
    }

    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse, DomainError> {
        let user = self
            .user_repo
            .find_by_username(&req.username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        let valid = self.hasher.verify(&req.password, &user.password_hash)?;
        if !valid {
            return Err(DomainError::InvalidCredentials);
        }

        let token = self
            .issuer
            .issue(&user.id.to_string(), &user.role.to_string())?;
        Ok(LoginResponse {
            token,
            user: UserResponse::from(user),
        })
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::application::services::fakes::{FakeHasher, FakeTokenIssuer, FakeUserRepo};
    use crate::domain::errors::DomainError;
    use crate::domain::user::{User, UserRole};

    fn make_user(username: &str, password: &str) -> User {
        User::new(
            username.into(),
            format!("hashed:{password}"),
            None,
            UserRole::Regular,
        )
    }

    fn svc(users: Vec<User>) -> AuthService {
        AuthService::new(
            Arc::new(FakeUserRepo::with(users)),
            Arc::new(FakeHasher),
            Arc::new(FakeTokenIssuer),
        )
    }

    #[tokio::test]
    async fn login_returns_token_for_valid_credentials() {
        let u = make_user("alice", "secret");
        let resp = svc(vec![u])
            .login(LoginRequest {
                username: "alice".into(),
                password: "secret".into(),
            })
            .await
            .unwrap();
        assert!(resp.token.starts_with("token:"));
        assert_eq!(resp.user.username, "alice");
    }

    #[tokio::test]
    async fn login_token_contains_user_id_and_role() {
        let u = make_user("carol", "pass");
        let uid = u.id.to_string();
        let resp = svc(vec![u])
            .login(LoginRequest {
                username: "carol".into(),
                password: "pass".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.token, format!("token:{uid}:regular"));
    }

    #[tokio::test]
    async fn login_fails_for_unknown_username() {
        let err = svc(vec![])
            .login(LoginRequest {
                username: "nobody".into(),
                password: "pass".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidCredentials));
    }

    #[tokio::test]
    async fn login_fails_for_wrong_password() {
        let u = make_user("bob", "correct");
        let err = svc(vec![u])
            .login(LoginRequest {
                username: "bob".into(),
                password: "wrong".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidCredentials));
    }
}
