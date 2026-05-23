use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::group::GroupFilter;
use crate::application::dto::pagination::PageParams;
use crate::application::dto::user::UserFilter;
use crate::application::repositories::group::GroupRepository;
use crate::application::repositories::user::UserRepository;
use crate::application::services::auth::TokenIssuer;
use crate::application::services::users::PasswordHasher;
use crate::domain::errors::DomainError;
use crate::domain::group::Group;
use crate::domain::user::User;

// ── FakeGroupRepository ───────────────────────────────────────────────────────

pub struct FakeGroupRepo {
    store: Mutex<HashMap<Uuid, Group>>,
}

impl FakeGroupRepo {
    pub fn empty() -> Self {
        Self { store: Mutex::new(HashMap::new()) }
    }

    pub fn with(groups: Vec<Group>) -> Self {
        Self {
            store: Mutex::new(groups.into_iter().map(|g| (g.id, g)).collect()),
        }
    }
}

#[async_trait]
impl GroupRepository for FakeGroupRepo {
    async fn find_all(
        &self,
        filter: &GroupFilter,
        page: &PageParams,
    ) -> Result<(Vec<Group>, u64), DomainError> {
        let store = self.store.lock().unwrap();
        let mut items: Vec<Group> = store
            .values()
            .filter(|g| {
                if let Some(ref s) = filter.search {
                    if !g.name.contains(s.as_str()) {
                        return false;
                    }
                }
                if let Some(ref st) = filter.status {
                    if &g.status != st {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        let total = items.len() as u64;
        let data = items
            .into_iter()
            .skip(page.offset() as usize)
            .take(page.clamped_limit() as usize)
            .collect();
        Ok((data, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Group, DomainError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DomainError::GroupNotFound(id))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Group>, DomainError> {
        Ok(self.store.lock().unwrap().values().find(|g| g.name == name).cloned())
    }

    async fn save(&self, group: &Group) -> Result<(), DomainError> {
        self.store.lock().unwrap().insert(group.id, group.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(&id);
        Ok(())
    }
}

// ── FakeUserRepository ────────────────────────────────────────────────────────

pub struct FakeUserRepo {
    store: Mutex<HashMap<Uuid, User>>,
}

impl FakeUserRepo {
    pub fn empty() -> Self {
        Self { store: Mutex::new(HashMap::new()) }
    }

    pub fn with(users: Vec<User>) -> Self {
        Self {
            store: Mutex::new(users.into_iter().map(|u| (u.id, u)).collect()),
        }
    }
}

#[async_trait]
impl UserRepository for FakeUserRepo {
    async fn find_all(
        &self,
        filter: &UserFilter,
        page: &PageParams,
    ) -> Result<(Vec<User>, u64), DomainError> {
        let store = self.store.lock().unwrap();
        let mut items: Vec<User> = store
            .values()
            .filter(|u| {
                if let Some(ref s) = filter.search {
                    if !u.username.contains(s.as_str()) {
                        return false;
                    }
                }
                if let Some(gid) = filter.group_id {
                    if u.group_id != Some(gid) {
                        return false;
                    }
                }
                if let Some(ref st) = filter.status {
                    if &u.status != st {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        items.sort_by(|a, b| a.username.cmp(&b.username));
        let total = items.len() as u64;
        let data = items
            .into_iter()
            .skip(page.offset() as usize)
            .take(page.clamped_limit() as usize)
            .collect();
        Ok((data, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DomainError::UserNotFound(id))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        Ok(self.store.lock().unwrap().values().find(|u| u.username == username).cloned())
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        self.store.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(&id);
        Ok(())
    }
}

// ── FakeHasher ────────────────────────────────────────────────────────────────

pub struct FakeHasher;

impl PasswordHasher for FakeHasher {
    fn hash(&self, password: &str) -> Result<String, DomainError> {
        Ok(format!("hashed:{password}"))
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, DomainError> {
        Ok(*hash == format!("hashed:{password}"))
    }

    fn generate_random(&self) -> String {
        "random-password-abc123xyz!".to_string()
    }
}

// ── FakeTokenIssuer ───────────────────────────────────────────────────────────

pub struct FakeTokenIssuer;

impl TokenIssuer for FakeTokenIssuer {
    fn issue(&self, user_id: &str, role: &str) -> Result<String, DomainError> {
        Ok(format!("token:{user_id}:{role}"))
    }
}
