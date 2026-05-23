use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for CollectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionStatus::Active => write!(f, "active"),
            CollectionStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl std::str::FromStr for CollectionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(CollectionStatus::Active),
            "inactive" => Ok(CollectionStatus::Inactive),
            other => Err(format!("unknown collection status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionVisibility {
    Private,
    Public,
}

impl std::fmt::Display for CollectionVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionVisibility::Private => write!(f, "private"),
            CollectionVisibility::Public => write!(f, "public"),
        }
    }
}

impl std::str::FromStr for CollectionVisibility {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(CollectionVisibility::Private),
            "public" => Ok(CollectionVisibility::Public),
            other => Err(format!("unknown collection visibility: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub status: CollectionStatus,
    pub visibility: CollectionVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Collection {
    pub fn new(
        name: String,
        description: Option<String>,
        owner_id: Uuid,
        visibility: CollectionVisibility,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            owner_id,
            status: CollectionStatus::Active,
            visibility,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        status: Option<CollectionStatus>,
        visibility: Option<CollectionVisibility>,
    ) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(d) = description {
            self.description = d;
        }
        if let Some(s) = status {
            self.status = s;
        }
        if let Some(v) = visibility {
            self.visibility = v;
        }
        self.updated_at = Utc::now();
    }

    pub fn transfer_ownership(&mut self, new_owner_id: Uuid) {
        self.owner_id = new_owner_id;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn new_sets_active_status_and_private_visibility_by_default() {
        let c = Collection::new("My API".into(), None, owner(), CollectionVisibility::Private);
        assert_eq!(c.status, CollectionStatus::Active);
        assert_eq!(c.visibility, CollectionVisibility::Private);
        assert!(c.description.is_none());
    }

    #[test]
    fn new_assigns_unique_ids() {
        let oid = owner();
        let a = Collection::new("A".into(), None, oid, CollectionVisibility::Private);
        let b = Collection::new("A".into(), None, oid, CollectionVisibility::Private);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_stores_public_visibility() {
        let c = Collection::new("Pub".into(), None, owner(), CollectionVisibility::Public);
        assert_eq!(c.visibility, CollectionVisibility::Public);
    }

    #[test]
    fn new_stores_description() {
        let c = Collection::new("C".into(), Some("desc".into()), owner(), CollectionVisibility::Private);
        assert_eq!(c.description.as_deref(), Some("desc"));
    }

    #[test]
    fn apply_update_name_only() {
        let mut c = Collection::new("Old".into(), None, owner(), CollectionVisibility::Private);
        let before = c.updated_at;
        c.apply_update(Some("New".into()), None, None, None);
        assert_eq!(c.name, "New");
        assert!(c.description.is_none());
        assert_eq!(c.status, CollectionStatus::Active);
        assert_eq!(c.visibility, CollectionVisibility::Private);
        assert!(c.updated_at >= before);
    }

    #[test]
    fn apply_update_clears_description_with_some_none() {
        let mut c = Collection::new("C".into(), Some("desc".into()), owner(), CollectionVisibility::Private);
        c.apply_update(None, Some(None), None, None);
        assert!(c.description.is_none());
    }

    #[test]
    fn apply_update_sets_description_with_some_some() {
        let mut c = Collection::new("C".into(), None, owner(), CollectionVisibility::Private);
        c.apply_update(None, Some(Some("new desc".into())), None, None);
        assert_eq!(c.description.as_deref(), Some("new desc"));
    }

    #[test]
    fn apply_update_status() {
        let mut c = Collection::new("C".into(), None, owner(), CollectionVisibility::Private);
        c.apply_update(None, None, Some(CollectionStatus::Inactive), None);
        assert_eq!(c.status, CollectionStatus::Inactive);
    }

    #[test]
    fn apply_update_visibility() {
        let mut c = Collection::new("C".into(), None, owner(), CollectionVisibility::Private);
        c.apply_update(None, None, None, Some(CollectionVisibility::Public));
        assert_eq!(c.visibility, CollectionVisibility::Public);
    }

    #[test]
    fn apply_update_none_args_leave_fields_unchanged() {
        let oid = owner();
        let mut c = Collection::new("C".into(), Some("d".into()), oid, CollectionVisibility::Public);
        c.apply_update(None, None, None, None);
        assert_eq!(c.name, "C");
        assert_eq!(c.description.as_deref(), Some("d"));
        assert_eq!(c.status, CollectionStatus::Active);
        assert_eq!(c.visibility, CollectionVisibility::Public);
        assert_eq!(c.owner_id, oid);
    }

    #[test]
    fn transfer_ownership_changes_owner_and_updates_timestamp() {
        let original_owner = owner();
        let new_owner = owner();
        let mut c = Collection::new("C".into(), None, original_owner, CollectionVisibility::Private);
        let before = c.updated_at;
        c.transfer_ownership(new_owner);
        assert_eq!(c.owner_id, new_owner);
        assert_ne!(c.owner_id, original_owner);
        assert!(c.updated_at >= before);
    }

    #[test]
    fn collection_status_display_roundtrip() {
        for (status, s) in [
            (CollectionStatus::Active, "active"),
            (CollectionStatus::Inactive, "inactive"),
        ] {
            assert_eq!(status.to_string(), s);
            assert_eq!(s.parse::<CollectionStatus>().unwrap(), status);
        }
    }

    #[test]
    fn collection_status_from_str_unknown_returns_err() {
        assert!("".parse::<CollectionStatus>().is_err());
        assert!("unknown".parse::<CollectionStatus>().is_err());
    }

    #[test]
    fn collection_visibility_display_roundtrip() {
        for (vis, s) in [
            (CollectionVisibility::Private, "private"),
            (CollectionVisibility::Public, "public"),
        ] {
            assert_eq!(vis.to_string(), s);
            assert_eq!(s.parse::<CollectionVisibility>().unwrap(), vis);
        }
    }

    #[test]
    fn collection_visibility_from_str_unknown_returns_err() {
        assert!("".parse::<CollectionVisibility>().is_err());
        assert!("shared".parse::<CollectionVisibility>().is_err());
    }
}
