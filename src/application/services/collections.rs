use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::collection::{
    CollectionFilter, CollectionResponse, CreateCollectionRequest, TransferOwnershipRequest,
    UpdateCollectionRequest,
};
use crate::application::dto::collection_share::{
    CollectionShareResponse, CreateShareRequest, UpdateShareRequest,
};
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::application::repositories::collection::CollectionRepository;
use crate::application::repositories::collection_share::CollectionShareRepository;
use crate::application::repositories::user::UserRepository;
use crate::domain::collection::{Collection, CollectionVisibility};
use crate::domain::collection_share::CollectionShare;
use crate::domain::errors::DomainError;

pub struct CollectionService {
    collection_repo: Arc<dyn CollectionRepository>,
    share_repo: Arc<dyn CollectionShareRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl CollectionService {
    pub fn new(
        collection_repo: Arc<dyn CollectionRepository>,
        share_repo: Arc<dyn CollectionShareRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self { collection_repo, share_repo, user_repo }
    }

    pub async fn list(
        &self,
        caller_id: Uuid,
        filter: CollectionFilter,
        page: PageParams,
    ) -> Result<Paginated<CollectionResponse>, DomainError> {
        let page = PageParams { limit: page.clamped_limit(), ..page };
        let caller = self.user_repo.find_by_id(caller_id).await?;
        let (collections, total) = self
            .collection_repo
            .find_all(caller_id, caller.group_id, &filter, &page)
            .await?;
        let data = collections.into_iter().map(CollectionResponse::from).collect();
        Ok(Paginated::new(data, total, &page))
    }

    pub async fn get(&self, id: Uuid, caller_id: Uuid) -> Result<CollectionResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !self.has_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        Ok(CollectionResponse::from(collection))
    }

    pub async fn create(
        &self,
        caller_id: Uuid,
        req: CreateCollectionRequest,
    ) -> Result<CollectionResponse, DomainError> {
        let visibility = req.visibility.unwrap_or(CollectionVisibility::Private);
        let collection = Collection::new(req.name, req.description, caller_id, visibility);
        self.collection_repo.save(&collection).await?;
        Ok(CollectionResponse::from(collection))
    }

    pub async fn update(
        &self,
        id: Uuid,
        caller_id: Uuid,
        req: UpdateCollectionRequest,
    ) -> Result<CollectionResponse, DomainError> {
        let mut collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        collection.apply_update(req.name, req.description, req.status, req.visibility);
        self.collection_repo.save(&collection).await?;
        Ok(CollectionResponse::from(collection))
    }

    pub async fn delete(&self, id: Uuid, caller_id: Uuid) -> Result<(), DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        self.share_repo.delete_by_collection(id).await?;
        self.collection_repo.delete(id).await
    }

    pub async fn duplicate(
        &self,
        id: Uuid,
        caller_id: Uuid,
    ) -> Result<CollectionResponse, DomainError> {
        let original = self.collection_repo.find_by_id(id).await?;
        if !self.has_access(&original, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let copy = Collection::new(
            format!("{} (copy)", original.name),
            original.description,
            caller_id,
            original.visibility,
        );
        self.collection_repo.save(&copy).await?;
        Ok(CollectionResponse::from(copy))
    }

    pub async fn list_shares(
        &self,
        id: Uuid,
        caller_id: Uuid,
    ) -> Result<Vec<CollectionShareResponse>, DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        let shares = self.share_repo.find_by_collection(id).await?;
        Ok(shares.into_iter().map(CollectionShareResponse::from).collect())
    }

    pub async fn add_share(
        &self,
        id: Uuid,
        caller_id: Uuid,
        req: CreateShareRequest,
    ) -> Result<CollectionShareResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        let share = match (req.user_id, req.group_id) {
            (Some(uid), None) => {
                if self.share_repo.find_existing(id, Some(uid), None).await?.is_some() {
                    return Err(DomainError::Conflict("already shared with this user".into()));
                }
                CollectionShare::new_user(id, uid, req.role)
            }
            (None, Some(gid)) => {
                if self.share_repo.find_existing(id, None, Some(gid)).await?.is_some() {
                    return Err(DomainError::Conflict("already shared with this group".into()));
                }
                CollectionShare::new_group(id, gid, req.role)
            }
            _ => {
                return Err(DomainError::InvalidInput(
                    "exactly one of user_id or group_id is required".into(),
                ))
            }
        };
        self.share_repo.save(&share).await?;
        Ok(CollectionShareResponse::from(share))
    }

    pub async fn update_share(
        &self,
        id: Uuid,
        share_id: Uuid,
        caller_id: Uuid,
        req: UpdateShareRequest,
    ) -> Result<CollectionShareResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        let mut share = self.share_repo.find_by_id(share_id).await?;
        share.apply_update(req.role);
        self.share_repo.save(&share).await?;
        Ok(CollectionShareResponse::from(share))
    }

    pub async fn remove_share(
        &self,
        id: Uuid,
        share_id: Uuid,
        caller_id: Uuid,
    ) -> Result<(), DomainError> {
        let collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        self.share_repo.find_by_id(share_id).await?;
        self.share_repo.delete(share_id).await
    }

    pub async fn transfer_ownership(
        &self,
        id: Uuid,
        caller_id: Uuid,
        req: TransferOwnershipRequest,
    ) -> Result<CollectionResponse, DomainError> {
        let mut collection = self.collection_repo.find_by_id(id).await?;
        if !Self::is_owner(&collection, caller_id) {
            return Err(DomainError::Forbidden);
        }
        // Verify the new owner exists before mutating the collection.
        self.user_repo.find_by_id(req.new_owner_id).await?;
        collection.transfer_ownership(req.new_owner_id);
        self.collection_repo.save(&collection).await?;
        Ok(CollectionResponse::from(collection))
    }

    fn is_owner(collection: &Collection, caller_id: Uuid) -> bool {
        collection.owner_id == caller_id
    }

    async fn has_access(
        &self,
        collection: &Collection,
        caller_id: Uuid,
    ) -> Result<bool, DomainError> {
        if collection.owner_id == caller_id {
            return Ok(true);
        }
        let caller = self.user_repo.find_by_id(caller_id).await?;
        let shares = self.share_repo.find_by_collection(collection.id).await?;
        Ok(shares.iter().any(|s| {
            s.user_id == Some(caller_id)
                || (caller.group_id.is_some() && s.group_id == caller.group_id)
        }))
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::dto::collection::{
        CollectionFilter, CreateCollectionRequest, TransferOwnershipRequest, UpdateCollectionRequest,
    };
    use crate::application::dto::collection_share::{CreateShareRequest, UpdateShareRequest};
    use crate::application::dto::pagination::PageParams;
    use crate::application::services::fakes::{
        FakeCollectionRepo, FakeCollectionShareRepo, FakeUserRepo,
    };
    use crate::domain::collection::{Collection, CollectionStatus, CollectionVisibility};
    use crate::domain::collection_share::{CollectionShare, ShareRole};
    use crate::domain::errors::DomainError;
    use crate::domain::user::{User, UserRole};

    fn make_user(username: &str) -> User {
        User::new(username.into(), "h".into(), None, UserRole::Regular)
    }

    fn make_collection(name: &str, owner_id: Uuid) -> Collection {
        Collection::new(name.into(), None, owner_id, CollectionVisibility::Private)
    }

    fn no_filter() -> CollectionFilter {
        CollectionFilter { search: None, status: None, visibility: None }
    }

    fn svc(
        caller: User,
        collections: Vec<Collection>,
        shares: Vec<CollectionShare>,
    ) -> (CollectionService, Uuid) {
        let caller_id = caller.id;
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(collections)),
            Arc::new(FakeCollectionShareRepo::with(shares)),
            Arc::new(FakeUserRepo::with(vec![caller])),
        );
        (service, caller_id)
    }

    fn empty_svc(caller: User) -> (CollectionService, Uuid) {
        svc(caller, vec![], vec![])
    }

    // ── list ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_empty_store_returns_empty_page() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let result = service.list(caller_id, no_filter(), PageParams::default()).await.unwrap();
        assert!(result.data.is_empty());
        assert_eq!(result.meta.total, 0);
    }

    #[tokio::test]
    async fn list_returns_only_owned_collections() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let other_id = Uuid::new_v4();
        let mine = make_collection("Mine", owner_id);
        let theirs = make_collection("Theirs", other_id);
        let (service, caller_id) = svc(owner, vec![mine, theirs], vec![]);
        let result = service.list(caller_id, no_filter(), PageParams::default()).await.unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "Mine");
    }

    #[tokio::test]
    async fn list_filters_by_search() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let (service, caller_id) = svc(
            owner,
            vec![make_collection("Alpha", owner_id), make_collection("Beta", owner_id)],
            vec![],
        );
        let result = service
            .list(
                caller_id,
                CollectionFilter { search: Some("Alp".into()), status: None, visibility: None },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "Alpha");
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let mut inactive = make_collection("Inactive", owner_id);
        inactive.status = CollectionStatus::Inactive;
        let active = make_collection("Active", owner_id);
        let (service, caller_id) = svc(owner, vec![inactive, active], vec![]);
        let result = service
            .list(
                caller_id,
                CollectionFilter {
                    search: None,
                    status: Some(CollectionStatus::Active),
                    visibility: None,
                },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "Active");
    }

    #[tokio::test]
    async fn list_filters_by_visibility() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let public = Collection::new("Public".into(), None, owner_id, CollectionVisibility::Public);
        let private = make_collection("Private", owner_id);
        let (service, caller_id) = svc(owner, vec![public, private], vec![]);
        let result = service
            .list(
                caller_id,
                CollectionFilter {
                    search: None,
                    status: None,
                    visibility: Some(CollectionVisibility::Public),
                },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "Public");
    }

    #[tokio::test]
    async fn list_returns_not_found_when_caller_does_not_exist() {
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::empty()),
        );
        let err = service.list(Uuid::new_v4(), no_filter(), PageParams::default()).await.unwrap_err();
        assert!(matches!(err, DomainError::UserNotFound(_)));
    }

    // ── get ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_returns_collection_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("My Collection", owner_id);
        let id = c.id;
        let (service, caller_id) = svc(owner, vec![c], vec![]);
        let resp = service.get(id, caller_id).await.unwrap();
        assert_eq!(resp.id, id);
        assert_eq!(resp.name, "My Collection");
    }

    #[tokio::test]
    async fn get_returns_collection_for_user_with_share() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("Shared Collection", owner.id);
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        // Build a fresh service with both users seeded so user lookup works for both owner and viewer.
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![CollectionShare::new_user(
                c.id, viewer_id, ShareRole::Viewer,
            )])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let resp = service.get(c.id, viewer_id).await.unwrap();
        assert_eq!(resp.id, c.id);
    }

    #[tokio::test]
    async fn get_returns_forbidden_when_caller_has_no_access() {
        let owner = make_user("alice");
        let stranger = make_user("charlie");
        let stranger_id = stranger.id;
        let c = make_collection("Private", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.get(c.id, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn get_returns_not_found_for_unknown_id() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let err = service.get(Uuid::new_v4(), caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    // ── create ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_sets_caller_as_owner() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let resp = service
            .create(
                caller_id,
                CreateCollectionRequest { name: "New".into(), description: None, visibility: None },
            )
            .await
            .unwrap();
        assert_eq!(resp.owner_id, caller_id);
        assert_eq!(resp.name, "New");
    }

    #[tokio::test]
    async fn create_defaults_visibility_to_private() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let resp = service
            .create(
                caller_id,
                CreateCollectionRequest { name: "C".into(), description: None, visibility: None },
            )
            .await
            .unwrap();
        assert_eq!(resp.visibility, CollectionVisibility::Private);
    }

    #[tokio::test]
    async fn create_stores_provided_visibility() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let resp = service
            .create(
                caller_id,
                CreateCollectionRequest {
                    name: "C".into(),
                    description: None,
                    visibility: Some(CollectionVisibility::Public),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.visibility, CollectionVisibility::Public);
    }

    #[tokio::test]
    async fn create_stores_description() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let resp = service
            .create(
                caller_id,
                CreateCollectionRequest {
                    name: "C".into(),
                    description: Some("My description".into()),
                    visibility: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.description.as_deref(), Some("My description"));
    }

    // ── update ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_changes_name_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("Old", owner_id);
        let id = c.id;
        let (service, caller_id) = svc(owner, vec![c], vec![]);
        let resp = service
            .update(
                id,
                caller_id,
                UpdateCollectionRequest {
                    name: Some("New".into()),
                    description: None,
                    status: None,
                    visibility: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "New");
    }

    #[tokio::test]
    async fn update_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service
            .update(
                c.id,
                stranger_id,
                UpdateCollectionRequest { name: Some("X".into()), description: None, status: None, visibility: None },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn update_returns_not_found_for_unknown_collection() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let err = service
            .update(
                Uuid::new_v4(),
                caller_id,
                UpdateCollectionRequest { name: None, description: None, status: None, visibility: None },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    // ── delete ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_removes_collection_and_its_shares() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let share = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![share]);
        service.delete(c.id, caller_id).await.unwrap();
        let err = service.get(c.id, caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    #[tokio::test]
    async fn delete_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.delete(c.id, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn delete_returns_not_found_for_unknown_collection() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let err = service.delete(Uuid::new_v4(), caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    // ── duplicate ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn duplicate_creates_copy_with_copy_suffix_in_name() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("Original", owner_id);
        let id = c.id;
        let (service, caller_id) = svc(owner, vec![c], vec![]);
        let copy = service.duplicate(id, caller_id).await.unwrap();
        assert_eq!(copy.name, "Original (copy)");
        assert_ne!(copy.id, id);
    }

    #[tokio::test]
    async fn duplicate_sets_caller_as_owner_of_copy() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let copy = service.duplicate(c.id, viewer_id).await.unwrap();
        assert_eq!(copy.owner_id, viewer_id);
    }

    #[tokio::test]
    async fn duplicate_returns_forbidden_when_caller_has_no_access() {
        let owner = make_user("alice");
        let stranger = make_user("charlie");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.duplicate(c.id, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn duplicate_returns_not_found_for_unknown_collection() {
        let caller = make_user("alice");
        let (service, caller_id) = empty_svc(caller);
        let err = service.duplicate(Uuid::new_v4(), caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    // ── list_shares ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_shares_returns_all_shares_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let s1 = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let s2 = CollectionShare::new_group(c.id, Uuid::new_v4(), ShareRole::Editor);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![s1, s2]);
        let shares = service.list_shares(c.id, caller_id).await.unwrap();
        assert_eq!(shares.len(), 2);
    }

    #[tokio::test]
    async fn list_shares_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.list_shares(c.id, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    // ── add_share ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn add_share_creates_user_share_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let target_user = Uuid::new_v4();
        let resp = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest { user_id: Some(target_user), group_id: None, role: ShareRole::Viewer },
            )
            .await
            .unwrap();
        assert_eq!(resp.user_id, Some(target_user));
        assert_eq!(resp.role, ShareRole::Viewer);
    }

    #[tokio::test]
    async fn add_share_creates_group_share_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let target_group = Uuid::new_v4();
        let resp = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest { user_id: None, group_id: Some(target_group), role: ShareRole::Editor },
            )
            .await
            .unwrap();
        assert_eq!(resp.group_id, Some(target_group));
        assert_eq!(resp.role, ShareRole::Editor);
    }

    #[tokio::test]
    async fn add_share_returns_conflict_for_duplicate_user_share() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let target = Uuid::new_v4();
        let existing = CollectionShare::new_user(c.id, target, ShareRole::Viewer);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![existing]);
        let err = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest { user_id: Some(target), group_id: None, role: ShareRole::Editor },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn add_share_returns_conflict_for_duplicate_group_share() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let target = Uuid::new_v4();
        let existing = CollectionShare::new_group(c.id, target, ShareRole::Viewer);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![existing]);
        let err = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest { user_id: None, group_id: Some(target), role: ShareRole::Editor },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn add_share_returns_invalid_input_when_both_ids_provided() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let err = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest {
                    user_id: Some(Uuid::new_v4()),
                    group_id: Some(Uuid::new_v4()),
                    role: ShareRole::Viewer,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn add_share_returns_invalid_input_when_neither_id_provided() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let err = service
            .add_share(
                c.id,
                caller_id,
                CreateShareRequest { user_id: None, group_id: None, role: ShareRole::Viewer },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn add_share_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service
            .add_share(
                c.id,
                stranger_id,
                CreateShareRequest { user_id: Some(Uuid::new_v4()), group_id: None, role: ShareRole::Viewer },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    // ── update_share ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_share_changes_role() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let share = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let share_id = share.id;
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![share]);
        let resp = service
            .update_share(c.id, share_id, caller_id, UpdateShareRequest { role: ShareRole::Editor })
            .await
            .unwrap();
        assert_eq!(resp.role, ShareRole::Editor);
    }

    #[tokio::test]
    async fn update_share_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let share = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let share_id = share.id;
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service
            .update_share(c.id, share_id, stranger_id, UpdateShareRequest { role: ShareRole::Editor })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn update_share_returns_not_found_for_unknown_share() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let err = service
            .update_share(c.id, Uuid::new_v4(), caller_id, UpdateShareRequest { role: ShareRole::Editor })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::CollectionShareNotFound(_)));
    }

    // ── remove_share ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn remove_share_deletes_the_share() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let share = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let share_id = share.id;
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![share]);
        service.remove_share(c.id, share_id, caller_id).await.unwrap();
        let remaining = service.list_shares(c.id, caller_id).await.unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn remove_share_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let share = CollectionShare::new_user(c.id, Uuid::new_v4(), ShareRole::Viewer);
        let share_id = share.id;
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.remove_share(c.id, share_id, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn remove_share_returns_not_found_for_unknown_share() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let err = service.remove_share(c.id, Uuid::new_v4(), caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionShareNotFound(_)));
    }

    // ── transfer_ownership ────────────────────────────────────────────────────

    #[tokio::test]
    async fn transfer_ownership_changes_owner() {
        let owner = make_user("alice");
        let new_owner = make_user("bob");
        let new_owner_id = new_owner.id;
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, new_owner])),
        );
        let resp = service
            .transfer_ownership(c.id, owner_id, TransferOwnershipRequest { new_owner_id })
            .await
            .unwrap();
        assert_eq!(resp.owner_id, new_owner_id);
    }

    #[tokio::test]
    async fn transfer_ownership_returns_forbidden_for_non_owner() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let new_owner = make_user("charlie");
        let new_owner_id = new_owner.id;
        let c = make_collection("C", owner.id);
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger, new_owner])),
        );
        let err = service
            .transfer_ownership(c.id, stranger_id, TransferOwnershipRequest { new_owner_id })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn transfer_ownership_returns_not_found_for_unknown_new_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (service, caller_id) = svc(owner, vec![c.clone()], vec![]);
        let err = service
            .transfer_ownership(c.id, caller_id, TransferOwnershipRequest { new_owner_id: Uuid::new_v4() })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::UserNotFound(_)));
    }

    #[tokio::test]
    async fn transfer_ownership_returns_not_found_for_unknown_collection() {
        let owner = make_user("alice");
        let new_owner = make_user("bob");
        let new_owner_id = new_owner.id;
        let owner_id = owner.id;
        let service = CollectionService::new(
            Arc::new(FakeCollectionRepo::with(vec![])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, new_owner])),
        );
        let err = service
            .transfer_ownership(Uuid::new_v4(), owner_id, TransferOwnershipRequest { new_owner_id })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }
}
