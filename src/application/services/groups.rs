use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::group::{
    CreateGroupRequest, GroupFilter, GroupResponse, UpdateGroupRequest,
};
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::application::repositories::group::GroupRepository;
use crate::domain::errors::DomainError;
use crate::domain::group::Group;

pub struct GroupService {
    repo: Arc<dyn GroupRepository>,
}

impl GroupService {
    pub fn new(repo: Arc<dyn GroupRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        filter: GroupFilter,
        page: PageParams,
    ) -> Result<Paginated<GroupResponse>, DomainError> {
        let page = PageParams {
            limit: page.clamped_limit(),
            ..page
        };
        let (groups, total) = self.repo.find_all(&filter, &page).await?;
        let data = groups.into_iter().map(GroupResponse::from).collect();
        Ok(Paginated::new(data, total, &page))
    }

    pub async fn get(&self, id: Uuid) -> Result<GroupResponse, DomainError> {
        self.repo.find_by_id(id).await.map(GroupResponse::from)
    }

    pub async fn create(&self, req: CreateGroupRequest) -> Result<GroupResponse, DomainError> {
        if self.repo.find_by_name(&req.name).await?.is_some() {
            return Err(DomainError::GroupNameTaken(req.name));
        }
        let group = Group::new(req.name, req.description);
        self.repo.save(&group).await?;
        Ok(GroupResponse::from(group))
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateGroupRequest,
    ) -> Result<GroupResponse, DomainError> {
        let mut group = self.repo.find_by_id(id).await?;

        if let Some(ref name) = req.name {
            if let Some(existing) = self.repo.find_by_name(name).await? {
                if existing.id != id {
                    return Err(DomainError::GroupNameTaken(name.clone()));
                }
            }
        }

        group.apply_update(req.name, req.description, req.status);
        self.repo.save(&group).await?;
        Ok(GroupResponse::from(group))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        self.repo.find_by_id(id).await?;
        self.repo.delete(id).await
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;
    use crate::application::dto::group::{CreateGroupRequest, GroupFilter, UpdateGroupRequest};
    use crate::application::dto::pagination::PageParams;
    use crate::application::services::fakes::FakeGroupRepo;
    use crate::domain::errors::DomainError;
    use crate::domain::group::{Group, GroupStatus};

    fn svc(groups: Vec<Group>) -> GroupService {
        GroupService::new(Arc::new(FakeGroupRepo::with(groups)))
    }

    fn empty() -> GroupService {
        GroupService::new(Arc::new(FakeGroupRepo::empty()))
    }

    #[tokio::test]
    async fn list_empty_store() {
        let result = empty()
            .list(
                GroupFilter {
                    search: None,
                    status: None,
                },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert!(result.data.is_empty());
        assert_eq!(result.meta.total, 0);
    }

    #[tokio::test]
    async fn list_returns_all_groups() {
        let result = svc(vec![
            Group::new("A".into(), None),
            Group::new("B".into(), None),
        ])
        .list(
            GroupFilter {
                search: None,
                status: None,
            },
            PageParams::default(),
        )
        .await
        .unwrap();
        assert_eq!(result.meta.total, 2);
        assert_eq!(result.data.len(), 2);
    }

    #[tokio::test]
    async fn list_filters_by_search() {
        let result = svc(vec![
            Group::new("Engineering".into(), None),
            Group::new("Marketing".into(), None),
        ])
        .list(
            GroupFilter {
                search: Some("Eng".into()),
                status: None,
            },
            PageParams::default(),
        )
        .await
        .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "Engineering");
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let mut inactive = Group::new("A".into(), None);
        inactive.status = GroupStatus::Inactive;
        let result = svc(vec![inactive, Group::new("B".into(), None)])
            .list(
                GroupFilter {
                    search: None,
                    status: Some(GroupStatus::Active),
                },
                PageParams::default(),
            )
            .await
            .unwrap();
        assert_eq!(result.meta.total, 1);
        assert_eq!(result.data[0].name, "B");
    }

    #[tokio::test]
    async fn get_returns_group_by_id() {
        let g = Group::new("Ops".into(), None);
        let id = g.id;
        let resp = svc(vec![g]).get(id).await.unwrap();
        assert_eq!(resp.id, id);
        assert_eq!(resp.name, "Ops");
    }

    #[tokio::test]
    async fn get_returns_not_found_for_unknown_id() {
        let err = empty().get(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, DomainError::GroupNotFound(_)));
    }

    #[tokio::test]
    async fn create_saves_new_group() {
        let resp = empty()
            .create(CreateGroupRequest {
                name: "DevOps".into(),
                description: None,
            })
            .await
            .unwrap();
        assert_eq!(resp.name, "DevOps");
    }

    #[tokio::test]
    async fn create_rejects_duplicate_name() {
        let g = Group::new("DevOps".into(), None);
        let err = svc(vec![g])
            .create(CreateGroupRequest {
                name: "DevOps".into(),
                description: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::GroupNameTaken(_)));
    }

    #[tokio::test]
    async fn update_changes_name() {
        let g = Group::new("OldName".into(), None);
        let id = g.id;
        let resp = svc(vec![g])
            .update(
                id,
                UpdateGroupRequest {
                    name: Some("NewName".into()),
                    description: None,
                    status: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "NewName");
    }

    #[tokio::test]
    async fn update_rejects_duplicate_name_from_other_group() {
        let g1 = Group::new("Alpha".into(), None);
        let g2 = Group::new("Beta".into(), None);
        let id2 = g2.id;
        let err = svc(vec![g1, g2])
            .update(
                id2,
                UpdateGroupRequest {
                    name: Some("Alpha".into()),
                    description: None,
                    status: None,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::GroupNameTaken(_)));
    }

    #[tokio::test]
    async fn update_allows_same_name_for_same_group() {
        let g = Group::new("Alpha".into(), None);
        let id = g.id;
        let resp = svc(vec![g])
            .update(
                id,
                UpdateGroupRequest {
                    name: Some("Alpha".into()),
                    description: None,
                    status: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.name, "Alpha");
    }

    #[tokio::test]
    async fn delete_removes_group() {
        let g = Group::new("ToDelete".into(), None);
        let id = g.id;
        let service = svc(vec![g]);
        service.delete(id).await.unwrap();
        assert!(matches!(
            service.get(id).await.unwrap_err(),
            DomainError::GroupNotFound(_)
        ));
    }

    #[tokio::test]
    async fn delete_returns_not_found_for_unknown_id() {
        let err = empty().delete(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, DomainError::GroupNotFound(_)));
    }
}
