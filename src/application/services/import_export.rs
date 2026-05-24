use std::sync::Arc;
use uuid::Uuid;

use crate::application::io::ImportedCollection;
use crate::application::repositories::collection::CollectionRepository;
use crate::application::repositories::collection_share::CollectionShareRepository;
use crate::application::repositories::endpoint::EndpointRepository;
use crate::domain::collection::{slugify_code, Collection, CollectionVisibility};
use crate::domain::endpoint::Endpoint;
use crate::domain::errors::DomainError;

pub struct ImportExportService {
    collection_repo: Arc<dyn CollectionRepository>,
    endpoint_repo: Arc<dyn EndpointRepository>,
    share_repo: Arc<dyn CollectionShareRepository>,
}

impl ImportExportService {
    pub fn new(
        collection_repo: Arc<dyn CollectionRepository>,
        endpoint_repo: Arc<dyn EndpointRepository>,
        share_repo: Arc<dyn CollectionShareRepository>,
    ) -> Self {
        Self {
            collection_repo,
            endpoint_repo,
            share_repo,
        }
    }

    pub async fn import(
        &self,
        owner_id: Uuid,
        imported: ImportedCollection,
    ) -> Result<Collection, DomainError> {
        if imported.name.trim().is_empty() {
            return Err(DomainError::InvalidInput(
                "collection name must not be empty".into(),
            ));
        }

        let code = slugify_code(&imported.name);
        let collection = Collection::new(
            imported.name,
            code,
            imported.description,
            owner_id,
            CollectionVisibility::Private,
        );
        self.collection_repo.save(&collection).await?;

        for ep in imported.endpoints {
            let endpoint = Endpoint::new(
                collection.id,
                ep.name,
                ep.method,
                ep.path,
                ep.status_code,
                ep.delay_ms,
                ep.response_headers,
                ep.response_body,
                ep.response_content_type,
            );
            self.endpoint_repo.save(&endpoint).await?;
        }

        Ok(collection)
    }

    pub async fn export(
        &self,
        collection_id: Uuid,
        caller_id: Uuid,
    ) -> Result<(Collection, Vec<Endpoint>), DomainError> {
        let collection = self.collection_repo.find_by_id(collection_id).await?;

        if !self.has_access(&collection, caller_id).await {
            return Err(DomainError::Forbidden);
        }

        let endpoints = self
            .endpoint_repo
            .find_all_by_collection(collection_id)
            .await?;
        Ok((collection, endpoints))
    }

    async fn has_access(&self, collection: &Collection, caller_id: Uuid) -> bool {
        if collection.owner_id == caller_id {
            return true;
        }
        if collection.visibility == CollectionVisibility::Public {
            return true;
        }
        self.share_repo
            .find_by_collection(collection.id)
            .await
            .unwrap_or_default()
            .iter()
            .any(|s| s.user_id == Some(caller_id))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::io::{ImportedCollection, ImportedEndpoint};
    use crate::application::services::fakes::{
        FakeCollectionRepo, FakeCollectionShareRepo, FakeEndpointRepo,
    };
    use crate::domain::collection::{Collection, CollectionVisibility};
    use crate::domain::collection_share::{CollectionShare, ShareRole};
    use crate::domain::endpoint::HttpMethod;

    fn svc(collections: Vec<Collection>, shares: Vec<CollectionShare>) -> ImportExportService {
        ImportExportService::new(
            Arc::new(FakeCollectionRepo::with(collections)),
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionShareRepo::with(shares)),
        )
    }

    fn make_collection(owner_id: Uuid) -> Collection {
        Collection::new("C".into(), "c".into(), None, owner_id, CollectionVisibility::Private)
    }

    fn make_imported(name: &str, endpoints: Vec<ImportedEndpoint>) -> ImportedCollection {
        ImportedCollection {
            name: name.into(),
            description: None,
            endpoints,
        }
    }

    fn make_imported_ep(method: HttpMethod, path: &str) -> ImportedEndpoint {
        ImportedEndpoint {
            name: "ep".into(),
            method,
            path: path.into(),
            status_code: 200,
            response_headers: None,
            response_body: None,
            response_content_type: None,
            delay_ms: 0,
        }
    }

    #[tokio::test]
    async fn import_creates_collection_with_caller_as_owner() {
        let owner = Uuid::new_v4();
        let svc = ImportExportService::new(
            Arc::new(FakeCollectionRepo::empty()),
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionShareRepo::empty()),
        );
        let imported = make_imported("API", vec![]);
        let c = svc.import(owner, imported).await.unwrap();
        assert_eq!(c.owner_id, owner);
        assert_eq!(c.name, "API");
    }

    #[tokio::test]
    async fn import_saves_endpoints_under_collection() {
        let owner = Uuid::new_v4();
        let ep_repo = Arc::new(FakeEndpointRepo::empty());
        let svc = ImportExportService::new(
            Arc::new(FakeCollectionRepo::empty()),
            ep_repo.clone(),
            Arc::new(FakeCollectionShareRepo::empty()),
        );
        let imported = make_imported(
            "API",
            vec![
                make_imported_ep(HttpMethod::Get, "/users"),
                make_imported_ep(HttpMethod::Post, "/users"),
            ],
        );
        let c = svc.import(owner, imported).await.unwrap();

        let saved = ep_repo.find_all_by_collection(c.id).await.unwrap();
        assert_eq!(saved.len(), 2);
    }

    #[tokio::test]
    async fn import_rejects_empty_name() {
        let owner = Uuid::new_v4();
        let svc = ImportExportService::new(
            Arc::new(FakeCollectionRepo::empty()),
            Arc::new(FakeEndpointRepo::empty()),
            Arc::new(FakeCollectionShareRepo::empty()),
        );
        let err = svc
            .import(owner, make_imported("  ", vec![]))
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn export_succeeds_for_owner() {
        let owner = Uuid::new_v4();
        let c = make_collection(owner);
        let svc = svc(vec![c.clone()], vec![]);
        let (col, _) = svc.export(c.id, owner).await.unwrap();
        assert_eq!(col.id, c.id);
    }

    #[tokio::test]
    async fn export_returns_forbidden_for_stranger() {
        let owner = Uuid::new_v4();
        let stranger = Uuid::new_v4();
        let c = make_collection(owner);
        let svc = svc(vec![c.clone()], vec![]);
        let err = svc.export(c.id, stranger).await.unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn export_succeeds_for_user_with_share() {
        let owner = Uuid::new_v4();
        let shared_user = Uuid::new_v4();
        let c = make_collection(owner);
        let share = CollectionShare::new_user(c.id, shared_user, ShareRole::Viewer);
        let svc = svc(vec![c.clone()], vec![share]);
        let (col, _) = svc.export(c.id, shared_user).await.unwrap();
        assert_eq!(col.id, c.id);
    }

    #[tokio::test]
    async fn export_succeeds_for_public_collection() {
        let owner = Uuid::new_v4();
        let stranger = Uuid::new_v4();
        let mut c = make_collection(owner);
        c.visibility = CollectionVisibility::Public;
        let svc = svc(vec![c.clone()], vec![]);
        let (col, _) = svc.export(c.id, stranger).await.unwrap();
        assert_eq!(col.id, c.id);
    }

    #[tokio::test]
    async fn export_returns_not_found_for_unknown_collection() {
        let caller = Uuid::new_v4();
        let svc = svc(vec![], vec![]);
        let err = svc.export(Uuid::new_v4(), caller).await.unwrap_err();
        assert!(matches!(err, DomainError::CollectionNotFound(_)));
    }
}
