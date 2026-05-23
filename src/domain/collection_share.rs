use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareRole {
    Viewer,
    Editor,
}

impl std::fmt::Display for ShareRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareRole::Viewer => write!(f, "viewer"),
            ShareRole::Editor => write!(f, "editor"),
        }
    }
}

impl std::str::FromStr for ShareRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "viewer" => Ok(ShareRole::Viewer),
            "editor" => Ok(ShareRole::Editor),
            other => Err(format!("unknown share role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionShare {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub role: ShareRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CollectionShare {
    pub fn new_user(collection_id: Uuid, user_id: Uuid, role: ShareRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            collection_id,
            user_id: Some(user_id),
            group_id: None,
            role,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_group(collection_id: Uuid, group_id: Uuid, role: ShareRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            collection_id,
            user_id: None,
            group_id: Some(group_id),
            role,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, role: ShareRole) {
        self.role = role;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_user_sets_user_id_and_clears_group_id() {
        let cid = Uuid::new_v4();
        let uid = Uuid::new_v4();
        let s = CollectionShare::new_user(cid, uid, ShareRole::Viewer);
        assert_eq!(s.collection_id, cid);
        assert_eq!(s.user_id, Some(uid));
        assert!(s.group_id.is_none());
        assert_eq!(s.role, ShareRole::Viewer);
    }

    #[test]
    fn new_group_sets_group_id_and_clears_user_id() {
        let cid = Uuid::new_v4();
        let gid = Uuid::new_v4();
        let s = CollectionShare::new_group(cid, gid, ShareRole::Editor);
        assert_eq!(s.collection_id, cid);
        assert_eq!(s.group_id, Some(gid));
        assert!(s.user_id.is_none());
        assert_eq!(s.role, ShareRole::Editor);
    }

    #[test]
    fn new_user_and_new_group_produce_unique_ids() {
        let cid = Uuid::new_v4();
        let a = CollectionShare::new_user(cid, Uuid::new_v4(), ShareRole::Viewer);
        let b = CollectionShare::new_group(cid, Uuid::new_v4(), ShareRole::Viewer);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn apply_update_changes_role_and_updates_timestamp() {
        let mut s = CollectionShare::new_user(Uuid::new_v4(), Uuid::new_v4(), ShareRole::Viewer);
        let before = s.updated_at;
        s.apply_update(ShareRole::Editor);
        assert_eq!(s.role, ShareRole::Editor);
        assert!(s.updated_at >= before);
    }

    #[test]
    fn share_role_display_roundtrip() {
        for (role, s) in [(ShareRole::Viewer, "viewer"), (ShareRole::Editor, "editor")] {
            assert_eq!(role.to_string(), s);
            assert_eq!(s.parse::<ShareRole>().unwrap(), role);
        }
    }

    #[test]
    fn share_role_from_str_unknown_returns_err() {
        assert!("".parse::<ShareRole>().is_err());
        assert!("owner".parse::<ShareRole>().is_err());
    }
}
