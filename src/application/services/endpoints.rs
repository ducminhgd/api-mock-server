use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::endpoint::{
    CreateEndpointRequest, EndpointFilter, EndpointResponse, UpdateEndpointRequest,
};
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::application::repositories::collection::CollectionRepository;
use crate::application::repositories::collection_share::CollectionShareRepository;
use crate::application::repositories::endpoint::EndpointRepository;
use crate::application::repositories::user::UserRepository;
use crate::domain::collection::Collection;
use crate::domain::collection_share::ShareRole;
use crate::domain::endpoint::Endpoint;
use crate::domain::errors::DomainError;

pub struct EndpointService {
    endpoint_repo: Arc<dyn EndpointRepository>,
    collection_repo: Arc<dyn CollectionRepository>,
    share_repo: Arc<dyn CollectionShareRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl EndpointService {
    pub fn new(
        endpoint_repo: Arc<dyn EndpointRepository>,
        collection_repo: Arc<dyn CollectionRepository>,
        share_repo: Arc<dyn CollectionShareRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            endpoint_repo,
            collection_repo,
            share_repo,
            user_repo,
        }
    }

    pub async fn list(
        &self,
        collection_id: Uuid,
        caller_id: Uuid,
        filter: EndpointFilter,
        page: PageParams,
    ) -> Result<Paginated<EndpointResponse>, DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_read_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let page = PageParams {
            limit: page.clamped_limit(),
            ..page
        };
        let (endpoints, total) = self
            .endpoint_repo
            .find_by_collection(collection_id, &filter, &page)
            .await?;
        let data = endpoints.into_iter().map(EndpointResponse::from).collect();
        Ok(Paginated::new(data, total, &page))
    }

    pub async fn get(
        &self,
        collection_id: Uuid,
        endpoint_id: Uuid,
        caller_id: Uuid,
    ) -> Result<EndpointResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_read_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let endpoint = self.endpoint_repo.find_by_id(endpoint_id).await?;
        Ok(EndpointResponse::from(endpoint))
    }

    pub async fn create(
        &self,
        collection_id: Uuid,
        caller_id: Uuid,
        req: CreateEndpointRequest,
    ) -> Result<EndpointResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_write_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let endpoint = Endpoint::new(
            collection_id,
            req.name,
            req.method,
            req.path,
            req.status_code.unwrap_or(200),
            req.delay_ms.unwrap_or(0),
            req.response_headers,
            req.response_body,
            req.response_content_type,
        );
        self.endpoint_repo.save(&endpoint).await?;
        Ok(EndpointResponse::from(endpoint))
    }

    pub async fn update(
        &self,
        collection_id: Uuid,
        endpoint_id: Uuid,
        caller_id: Uuid,
        req: UpdateEndpointRequest,
    ) -> Result<EndpointResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_write_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let mut endpoint = self.endpoint_repo.find_by_id(endpoint_id).await?;
        endpoint.apply_update(
            req.name,
            req.method,
            req.path,
            req.status_code,
            req.delay_ms,
            req.response_headers,
            req.response_body,
            req.response_content_type,
            req.status,
        );
        self.endpoint_repo.save(&endpoint).await?;
        Ok(EndpointResponse::from(endpoint))
    }

    pub async fn delete(
        &self,
        collection_id: Uuid,
        endpoint_id: Uuid,
        caller_id: Uuid,
    ) -> Result<(), DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_write_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        self.endpoint_repo.find_by_id(endpoint_id).await?;
        self.endpoint_repo.delete(endpoint_id).await
    }

    pub async fn duplicate(
        &self,
        collection_id: Uuid,
        endpoint_id: Uuid,
        caller_id: Uuid,
    ) -> Result<EndpointResponse, DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;
        if !self.has_write_access(&collection, caller_id).await? {
            return Err(DomainError::Forbidden);
        }
        let original = self.endpoint_repo.find_by_id(endpoint_id).await?;
        let mut copy = original.copy_to(collection_id);
        copy.name = format!("{} (copy)", original.name);
        self.endpoint_repo.save(&copy).await?;
        Ok(EndpointResponse::from(copy))
    }

    fn is_owner(collection: &Collection, caller_id: Uuid) -> bool {
        collection.owner_id == caller_id
    }

    async fn has_read_access(
        &self,
        collection: &Collection,
        caller_id: Uuid,
    ) -> Result<bool, DomainError> {
        if Self::is_owner(collection, caller_id) {
            return Ok(true);
        }
        let caller = self.user_repo.find_by_id(caller_id).await?;
        let shares = self.share_repo.find_by_collection(collection.id).await?;
        Ok(shares.iter().any(|s| {
            s.user_id == Some(caller_id)
                || (caller.group_id.is_some() && s.group_id == caller.group_id)
        }))
    }

    async fn has_write_access(
        &self,
        collection: &Collection,
        caller_id: Uuid,
    ) -> Result<bool, DomainError> {
        if Self::is_owner(collection, caller_id) {
            return Ok(true);
        }
        let caller = self.user_repo.find_by_id(caller_id).await?;
        let shares = self.share_repo.find_by_collection(collection.id).await?;
        Ok(shares.iter().any(|s| {
            s.role == ShareRole::Editor
                && (s.user_id == Some(caller_id)
                    || (caller.group_id.is_some() && s.group_id == caller.group_id))
        }))
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::dto::endpoint::{
        CreateEndpointRequest, EndpointFilter, UpdateEndpointRequest,
    };
    use crate::application::dto::pagination::PageParams;
    use crate::application::services::fakes::{
        FakeCollectionRepo, FakeCollectionShareRepo, FakeEndpointRepo, FakeUserRepo,
    };
    use crate::domain::collection::{Collection, CollectionVisibility};
    use crate::domain::collection_share::{CollectionShare, ShareRole};
    use crate::domain::endpoint::{Endpoint, EndpointStatus, HttpMethod};
    use crate::domain::errors::DomainError;
    use crate::domain::user::{User, UserRole};

    fn make_user(username: &str) -> User {
        User::new(username.into(), "h".into(), None, UserRole::Regular)
    }

    fn make_collection(name: &str, owner_id: Uuid) -> Collection {
        Collection::new(name.into(), None, owner_id, CollectionVisibility::Private)
    }

    fn make_endpoint(name: &str, collection_id: Uuid) -> Endpoint {
        Endpoint::new(
            collection_id,
            name.into(),
            HttpMethod::Get,
            format!("/{name}"),
            200,
            0,
            None,
            None,
            None,
        )
    }

    fn no_filter() -> EndpointFilter {
        EndpointFilter::default()
    }

    fn svc(
        caller: User,
        collections: Vec<Collection>,
        endpoints: Vec<Endpoint>,
        shares: Vec<CollectionShare>,
    ) -> (EndpointService, Uuid) {
        let caller_id = caller.id;
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(endpoints)),
            Arc::new(FakeCollectionRepo::with(collections)),
            Arc::new(FakeCollectionShareRepo::with(shares)),
            Arc::new(FakeUserRepo::with(vec![caller])),
        );
        (service, caller_id)
    }

    // ── list ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_returns_endpoints_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let e1 = make_endpoint("Alpha", c.id);
        let e2 = make_endpoint("Beta", c.id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![e1, e2], vec![]);
        let result = svc
            .list(c.id, caller_id, no_filter(), PageParams::default())
            .await
            .unwrap();
        assert_eq!(result.meta.total, 2);
    }

    #[tokio::test]
    async fn list_returns_forbidden_for_no_access() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service
            .list(c.id, stranger_id, no_filter(), PageParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn list_returns_not_found_for_unknown_collection() {
        let caller = make_user("alice");
        let caller_id = caller.id;
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![caller])),
        );
        let err = service
            .list(
                Uuid::new_v4(),
                caller_id,
                no_filter(),
                PageParams::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    #[tokio::test]
    async fn list_returns_endpoints_for_viewer_share() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let result = service
            .list(c.id, viewer_id, no_filter(), PageParams::default())
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
    }

    #[tokio::test]
    async fn list_filters_by_method() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let get_ep = make_endpoint("GET-EP", c.id);
        let mut post_ep = make_endpoint("POST-EP", c.id);
        post_ep.method = HttpMethod::Post;
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![get_ep, post_ep], vec![]);
        let result = svc
            .list(
                c.id,
                caller_id,
                EndpointFilter {
                    method: Some(HttpMethod::Post),
                    ..Default::default()
                },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].method, HttpMethod::Post);
    }

    // ── get ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_returns_endpoint_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![e], vec![]);
        let resp = svc.get(c.id, eid, caller_id).await.unwrap();
        assert_eq!(resp.id, eid);
        assert_eq!(resp.name, "EP");
    }

    #[tokio::test]
    async fn get_returns_forbidden_for_no_access() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service.get(c.id, eid, stranger_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn get_returns_not_found_for_unknown_endpoint() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let err = svc.get(c.id, Uuid::new_v4(), caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::EndpointNotFound(_)));
    }

    // ── create ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_stores_endpoint_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let resp = svc
            .create(
                c.id,
                caller_id,
                CreateEndpointRequest {
                    name: "New EP".into(),
                    method: HttpMethod::Post,
                    path: "/items".into(),
                    status_code: Some(201),
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "New EP");
        assert_eq!(resp.method, HttpMethod::Post);
        assert_eq!(resp.status_code, 201);
        assert_eq!(resp.collection_id, c.id);
    }

    #[tokio::test]
    async fn create_defaults_status_code_to_200_and_delay_to_0() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let resp = svc
            .create(
                c.id,
                caller_id,
                CreateEndpointRequest {
                    name: "EP".into(),
                    method: HttpMethod::Get,
                    path: "/".into(),
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.delay_ms, 0);
    }

    #[tokio::test]
    async fn create_returns_forbidden_for_viewer() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let err = service
            .create(
                c.id,
                viewer_id,
                CreateEndpointRequest {
                    name: "EP".into(),
                    method: HttpMethod::Get,
                    path: "/".into(),
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn create_returns_forbidden_for_no_access() {
        let owner = make_user("alice");
        let stranger = make_user("bob");
        let stranger_id = stranger.id;
        let c = make_collection("C", owner.id);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![owner, stranger])),
        );
        let err = service
            .create(
                c.id,
                stranger_id,
                CreateEndpointRequest {
                    name: "EP".into(),
                    method: HttpMethod::Get,
                    path: "/".into(),
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn create_allowed_for_editor_share() {
        let owner = make_user("alice");
        let editor = make_user("bob");
        let editor_id = editor.id;
        let c = make_collection("C", owner.id);
        let share = CollectionShare::new_user(c.id, editor_id, ShareRole::Editor);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, editor])),
        );
        let resp = service
            .create(
                c.id,
                editor_id,
                CreateEndpointRequest {
                    name: "EP".into(),
                    method: HttpMethod::Get,
                    path: "/".into(),
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "EP");
    }

    // ── update ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn update_changes_name_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let e = make_endpoint("Old", c.id);
        let eid = e.id;
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![e], vec![]);
        let resp = svc
            .update(
                c.id,
                eid,
                caller_id,
                UpdateEndpointRequest {
                    name: Some("New".into()),
                    method: None,
                    path: None,
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                    status: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "New");
    }

    #[tokio::test]
    async fn update_returns_forbidden_for_viewer() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let err = service
            .update(
                c.id,
                eid,
                viewer_id,
                UpdateEndpointRequest {
                    name: Some("X".into()),
                    method: None,
                    path: None,
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                    status: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn update_returns_not_found_for_unknown_endpoint() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let err = svc
            .update(
                c.id,
                Uuid::new_v4(),
                caller_id,
                UpdateEndpointRequest {
                    name: None,
                    method: None,
                    path: None,
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                    status: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::EndpointNotFound(_)));
    }

    #[tokio::test]
    async fn update_returns_not_found_for_unknown_collection() {
        let caller = make_user("alice");
        let caller_id = caller.id;
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionRepo::with(vec![])),
            Arc::new(FakeCollectionShareRepo::empty()),
            Arc::new(FakeUserRepo::with(vec![caller])),
        );
        let err = service
            .update(
                Uuid::new_v4(),
                Uuid::new_v4(),
                caller_id,
                UpdateEndpointRequest {
                    name: None,
                    method: None,
                    path: None,
                    status_code: None,
                    delay_ms: None,
                    response_headers: None,
                    response_body: None,
                    response_content_type: None,
                    status: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }

    // ── delete ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_removes_endpoint_for_owner() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![e], vec![]);
        svc.delete(c.id, eid, caller_id).await.unwrap();
        let err = svc.get(c.id, eid, caller_id).await.unwrap_err();
        assert!(matches!(err, DomainError::EndpointNotFound(_)));
    }

    #[tokio::test]
    async fn delete_returns_forbidden_for_viewer() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let err = service.delete(c.id, eid, viewer_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn delete_returns_not_found_for_unknown_endpoint() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let err = svc
            .delete(c.id, Uuid::new_v4(), caller_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::EndpointNotFound(_)));
    }

    // ── duplicate ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn duplicate_creates_copy_with_copy_suffix() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let e = make_endpoint("Original", c.id);
        let eid = e.id;
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![e], vec![]);
        let copy = svc.duplicate(c.id, eid, caller_id).await.unwrap();
        assert_eq!(copy.name, "Original (copy)");
        assert_ne!(copy.id, eid);
        assert_eq!(copy.collection_id, c.id);
    }

    #[tokio::test]
    async fn duplicate_returns_forbidden_for_viewer() {
        let owner = make_user("alice");
        let viewer = make_user("bob");
        let viewer_id = viewer.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let share = CollectionShare::new_user(c.id, viewer_id, ShareRole::Viewer);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, viewer])),
        );
        let err = service.duplicate(c.id, eid, viewer_id).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn duplicate_allowed_for_editor() {
        let owner = make_user("alice");
        let editor = make_user("bob");
        let editor_id = editor.id;
        let c = make_collection("C", owner.id);
        let e = make_endpoint("EP", c.id);
        let eid = e.id;
        let share = CollectionShare::new_user(c.id, editor_id, ShareRole::Editor);
        let service = EndpointService::new(
            Arc::new(FakeEndpointRepo::with(vec![e])),
            Arc::new(FakeCollectionRepo::with(vec![c.clone()])),
            Arc::new(FakeCollectionShareRepo::with(vec![share])),
            Arc::new(FakeUserRepo::with(vec![owner, editor])),
        );
        let copy = service.duplicate(c.id, eid, editor_id).await.unwrap();
        assert_eq!(copy.name, "EP (copy)");
    }

    #[tokio::test]
    async fn duplicate_returns_not_found_for_unknown_endpoint() {
        let owner = make_user("alice");
        let owner_id = owner.id;
        let c = make_collection("C", owner_id);
        let (svc, caller_id) = svc(owner, vec![c.clone()], vec![], vec![]);
        let err = svc
            .duplicate(c.id, Uuid::new_v4(), caller_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::EndpointNotFound(_)));
    }
}
