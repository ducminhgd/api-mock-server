use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::dto::collection::CollectionFilter;
use crate::application::dto::endpoint::EndpointFilter;
use crate::application::dto::group::GroupFilter;
use crate::application::dto::pagination::PageParams;
use crate::application::dto::user::UserFilter;
use crate::application::repositories::collection::CollectionRepository;
use crate::application::repositories::collection_share::CollectionShareRepository;
use crate::application::repositories::endpoint::EndpointRepository;
use crate::application::repositories::group::GroupRepository;
use crate::application::repositories::user::UserRepository;
use crate::application::services::auth::TokenIssuer;
use crate::application::services::users::PasswordHasher;
use crate::domain::collection::Collection;
use crate::domain::collection_share::CollectionShare;
use crate::domain::endpoint::Endpoint;
use crate::domain::errors::DomainError;
use crate::domain::group::Group;
use crate::domain::user::User;

// ── FakeCollectionRepository ──────────────────────────────────────────────────
// Note: find_all only filters by owner_id (not shares). The SQL impl does the full
// access-scoped join. Shares-based visibility is covered by integration tests.

pub struct FakeCollectionRepo {
    store: Mutex<HashMap<Uuid, Collection>>,
}

impl FakeCollectionRepo {
    pub fn empty() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn with(collections: Vec<Collection>) -> Self {
        Self {
            store: Mutex::new(collections.into_iter().map(|c| (c.id, c)).collect()),
        }
    }
}

#[async_trait]
impl CollectionRepository for FakeCollectionRepo {
    async fn find_all(
        &self,
        caller_id: Uuid,
        _caller_group_id: Option<Uuid>,
        filter: &CollectionFilter,
        page: &PageParams,
    ) -> Result<(Vec<Collection>, u64), DomainError> {
        let store = self.store.lock().unwrap();
        let mut items: Vec<Collection> = store
            .values()
            .filter(|c| {
                if c.owner_id != caller_id {
                    return false;
                }
                if let Some(ref s) = filter.search {
                    if !c.name.contains(s.as_str()) {
                        return false;
                    }
                }
                if let Some(ref st) = filter.status {
                    if &c.status != st {
                        return false;
                    }
                }
                if let Some(ref v) = filter.visibility {
                    if &c.visibility != v {
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

    async fn find_by_id(&self, id: Uuid) -> Result<Collection, DomainError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DomainError::CollectionNotFound(id))
    }

    async fn save(&self, collection: &Collection) -> Result<(), DomainError> {
        self.store
            .lock()
            .unwrap()
            .insert(collection.id, collection.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(&id);
        Ok(())
    }
}

// ── FakeCollectionShareRepository ─────────────────────────────────────────────

pub struct FakeCollectionShareRepo {
    store: Mutex<HashMap<Uuid, CollectionShare>>,
}

impl FakeCollectionShareRepo {
    pub fn empty() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn with(shares: Vec<CollectionShare>) -> Self {
        Self {
            store: Mutex::new(shares.into_iter().map(|s| (s.id, s)).collect()),
        }
    }
}

#[async_trait]
impl CollectionShareRepository for FakeCollectionShareRepo {
    async fn find_by_collection(
        &self,
        collection_id: Uuid,
    ) -> Result<Vec<CollectionShare>, DomainError> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.collection_id == collection_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<CollectionShare, DomainError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DomainError::CollectionShareNotFound(id))
    }

    async fn find_existing(
        &self,
        collection_id: Uuid,
        user_id: Option<Uuid>,
        group_id: Option<Uuid>,
    ) -> Result<Option<CollectionShare>, DomainError> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|s| {
                s.collection_id == collection_id && s.user_id == user_id && s.group_id == group_id
            })
            .cloned())
    }

    async fn save(&self, share: &CollectionShare) -> Result<(), DomainError> {
        self.store.lock().unwrap().insert(share.id, share.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError> {
        self.store
            .lock()
            .unwrap()
            .retain(|_, s| s.collection_id != collection_id);
        Ok(())
    }
}

// ── FakeEndpointRepository ────────────────────────────────────────────────────

pub struct FakeEndpointRepo {
    store: Mutex<HashMap<Uuid, Endpoint>>,
}

impl FakeEndpointRepo {
    pub fn empty() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn with(endpoints: Vec<Endpoint>) -> Self {
        Self {
            store: Mutex::new(endpoints.into_iter().map(|e| (e.id, e)).collect()),
        }
    }
}

#[async_trait]
impl EndpointRepository for FakeEndpointRepo {
    async fn find_by_collection(
        &self,
        collection_id: Uuid,
        filter: &EndpointFilter,
        page: &PageParams,
    ) -> Result<(Vec<Endpoint>, u64), DomainError> {
        let store = self.store.lock().unwrap();
        let mut items: Vec<Endpoint> = store
            .values()
            .filter(|e| {
                if e.collection_id != collection_id {
                    return false;
                }
                if let Some(ref s) = filter.search {
                    if !e.name.contains(s.as_str()) && !e.path.contains(s.as_str()) {
                        return false;
                    }
                }
                if let Some(ref m) = filter.method {
                    if &e.method != m {
                        return false;
                    }
                }
                if let Some(ref st) = filter.status {
                    if &e.status != st {
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

    async fn find_all_by_collection(
        &self,
        collection_id: Uuid,
    ) -> Result<Vec<Endpoint>, DomainError> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.collection_id == collection_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Endpoint, DomainError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(DomainError::EndpointNotFound(id))
    }

    async fn save(&self, endpoint: &Endpoint) -> Result<(), DomainError> {
        self.store
            .lock()
            .unwrap()
            .insert(endpoint.id, endpoint.clone());
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.store.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn delete_by_collection(&self, collection_id: Uuid) -> Result<(), DomainError> {
        self.store
            .lock()
            .unwrap()
            .retain(|_, e| e.collection_id != collection_id);
        Ok(())
    }
}

// ── FakeGroupRepository ───────────────────────────────────────────────────────

pub struct FakeGroupRepo {
    store: Mutex<HashMap<Uuid, Group>>,
}

impl FakeGroupRepo {
    pub fn empty() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
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
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|g| g.name == name)
            .cloned())
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
        Self {
            store: Mutex::new(HashMap::new()),
        }
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
        Ok(self
            .store
            .lock()
            .unwrap()
            .values()
            .find(|u| u.username == username)
            .cloned())
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
