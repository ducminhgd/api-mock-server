use std::sync::Arc;
use uuid::Uuid;

use crate::application::repositories::collection::CollectionRepository;
use crate::application::repositories::endpoint::EndpointRepository;
use crate::domain::collection::CollectionStatus;
use crate::domain::endpoint::{EndpointStatus, HttpMethod};
use crate::domain::errors::DomainError;
use crate::domain::mock::{resolve_endpoint, MockResolution};

#[derive(Debug)]
pub struct MockResult {
    pub status_code: u16,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_content_type: Option<String>,
    pub delay_ms: u32,
}

#[derive(Debug)]
pub enum MockError {
    CollectionNotFound,
    ServiceUnavailable,
    NotFound,
    MethodNotAllowed(Vec<HttpMethod>),
    Internal(String),
}

pub struct MockService {
    collection_repo: Arc<dyn CollectionRepository>,
    endpoint_repo: Arc<dyn EndpointRepository>,
}

impl MockService {
    pub fn new(
        collection_repo: Arc<dyn CollectionRepository>,
        endpoint_repo: Arc<dyn EndpointRepository>,
    ) -> Self {
        Self {
            collection_repo,
            endpoint_repo,
        }
    }

    pub async fn resolve_by_code(
        &self,
        collection_code: &str,
        method: HttpMethod,
        path: &str,
    ) -> Result<MockResult, MockError> {
        let collection = self
            .collection_repo
            .find_by_code(collection_code)
            .await
            .map_err(|e| match e {
                DomainError::InvalidInput(_) => MockError::CollectionNotFound,
                _ => MockError::CollectionNotFound,
            })?;
        self.resolve_collection(collection.id, method, path).await
    }

    pub async fn resolve(
        &self,
        collection_id: Uuid,
        method: HttpMethod,
        path: &str,
    ) -> Result<MockResult, MockError> {
        self.resolve_collection(collection_id, method, path).await
    }

    async fn resolve_collection(
        &self,
        collection_id: Uuid,
        method: HttpMethod,
        path: &str,
    ) -> Result<MockResult, MockError> {
        let collection = self
            .collection_repo
            .find_by_id(collection_id)
            .await
            .map_err(|_| MockError::CollectionNotFound)?;

        if collection.status == CollectionStatus::Inactive {
            return Err(MockError::ServiceUnavailable);
        }

        let all_endpoints = self
            .endpoint_repo
            .find_all_by_collection(collection_id)
            .await
            .map_err(|e| MockError::Internal(e.to_string()))?;

        let active: Vec<_> = all_endpoints
            .into_iter()
            .filter(|ep| ep.status == EndpointStatus::Active)
            .collect();

        match resolve_endpoint(&active, &method, path) {
            MockResolution::NotFound => Err(MockError::NotFound),
            MockResolution::MethodNotAllowed(allowed) => Err(MockError::MethodNotAllowed(allowed)),
            MockResolution::Matched(ep) => Ok(MockResult {
                status_code: ep.status_code,
                response_headers: ep.response_headers.clone(),
                response_body: ep.response_body.clone(),
                response_content_type: ep.response_content_type.clone(),
                delay_ms: ep.delay_ms,
            }),
        }
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::services::fakes::{FakeCollectionRepo, FakeEndpointRepo};
    use crate::domain::collection::{Collection, CollectionStatus, CollectionVisibility};
    use crate::domain::endpoint::{Endpoint, EndpointStatus, HttpMethod};

    fn make_collection(owner_id: Uuid) -> Collection {
        Collection::new("C".into(), "c".into(), None, owner_id, CollectionVisibility::Public)
    }

    fn make_endpoint(collection_id: Uuid, method: HttpMethod, path: &str) -> Endpoint {
        Endpoint::new(
            collection_id,
            "EP".into(),
            method,
            path.into(),
            200,
            0,
            None,
            None,
            None,
        )
    }

    fn svc(collections: Vec<Collection>, endpoints: Vec<Endpoint>) -> MockService {
        MockService::new(
            Arc::new(FakeCollectionRepo::with(collections)),
            Arc::new(FakeEndpointRepo::with(endpoints)),
        )
    }

    #[tokio::test]
    async fn resolve_returns_result_for_active_collection_and_endpoint() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let ep = make_endpoint(c.id, HttpMethod::Get, "/users");
        let service = svc(vec![c.clone()], vec![ep]);
        let result = service
            .resolve(c.id, HttpMethod::Get, "/users")
            .await
            .unwrap();
        assert_eq!(result.status_code, 200);
    }

    #[tokio::test]
    async fn resolve_returns_service_unavailable_for_inactive_collection() {
        let owner = Uuid::new_v4();
        let mut c = make_collection(owner);
        c.status = CollectionStatus::Inactive;
        let service = svc(vec![c.clone()], vec![]);
        let err = service
            .resolve(c.id, HttpMethod::Get, "/users")
            .await
            .unwrap_err();
        assert!(matches!(err, MockError::ServiceUnavailable));
    }

    #[tokio::test]
    async fn resolve_returns_collection_not_found_for_unknown_id() {
        let service = svc(vec![], vec![]);
        let err = service
            .resolve(Uuid::new_v4(), HttpMethod::Get, "/users")
            .await
            .unwrap_err();
        assert!(matches!(err, MockError::CollectionNotFound));
    }

    #[tokio::test]
    async fn resolve_returns_not_found_when_no_endpoint_matches_path() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let ep = make_endpoint(c.id, HttpMethod::Get, "/orders");
        let service = svc(vec![c.clone()], vec![ep]);
        let err = service
            .resolve(c.id, HttpMethod::Get, "/users")
            .await
            .unwrap_err();
        assert!(matches!(err, MockError::NotFound));
    }

    #[tokio::test]
    async fn resolve_returns_method_not_allowed_when_path_matches_wrong_method() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let ep = make_endpoint(c.id, HttpMethod::Post, "/users");
        let service = svc(vec![c.clone()], vec![ep]);
        let err = service
            .resolve(c.id, HttpMethod::Get, "/users")
            .await
            .unwrap_err();
        assert!(matches!(err, MockError::MethodNotAllowed(_)));
    }

    #[tokio::test]
    async fn resolve_skips_inactive_endpoints() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let mut ep = make_endpoint(c.id, HttpMethod::Get, "/users");
        ep.status = EndpointStatus::Inactive;
        let service = svc(vec![c.clone()], vec![ep]);
        let err = service
            .resolve(c.id, HttpMethod::Get, "/users")
            .await
            .unwrap_err();
        assert!(matches!(err, MockError::NotFound));
    }

    #[tokio::test]
    async fn resolve_prefers_exact_over_wildcard() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let exact_ep = make_endpoint(c.id, HttpMethod::Get, "/users/me");
        let wild_ep = make_endpoint(c.id, HttpMethod::Get, "/users/{id}");
        let service = svc(vec![c.clone()], vec![wild_ep, exact_ep]);
        let result = service
            .resolve(c.id, HttpMethod::Get, "/users/me")
            .await
            .unwrap();
        // Verify the exact endpoint's status_code (both are 200, so just check it resolves)
        assert_eq!(result.status_code, 200);
        // Confirm by checking exact endpoint is matched (via a different status code trick)
        let owner2 = Uuid::new_v4();
        let c2 = make_collection(owner2);
        let mut exact2 = make_endpoint(c2.id, HttpMethod::Get, "/users/me");
        exact2.status_code = 201;
        let mut wild2 = make_endpoint(c2.id, HttpMethod::Get, "/users/{id}");
        wild2.status_code = 202;
        let service2 = svc(vec![c2.clone()], vec![wild2, exact2]);
        let r2 = service2
            .resolve(c2.id, HttpMethod::Get, "/users/me")
            .await
            .unwrap();
        assert_eq!(r2.status_code, 201);
    }

    #[tokio::test]
    async fn resolve_returns_configured_status_code_and_body() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let mut ep = make_endpoint(c.id, HttpMethod::Get, "/ping");
        ep.status_code = 204;
        ep.response_body = Some("pong".into());
        ep.delay_ms = 50;
        let service = svc(vec![c.clone()], vec![ep]);
        let result = service
            .resolve(c.id, HttpMethod::Get, "/ping")
            .await
            .unwrap();
        assert_eq!(result.status_code, 204);
        assert_eq!(result.response_body.as_deref(), Some("pong"));
        assert_eq!(result.delay_ms, 50);
    }
}
