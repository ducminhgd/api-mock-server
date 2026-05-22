use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Regular,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Regular => write!(f, "regular"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "regular" => Ok(UserRole::Regular),
            other => Err(format!("unknown user role: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            other => Err(format!("unknown user status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub group_id: Option<Uuid>,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        username: String,
        password_hash: String,
        group_id: Option<Uuid>,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            group_id,
            role,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn apply_update(
        &mut self,
        username: Option<String>,
        group_id: Option<Option<Uuid>>,
        status: Option<UserStatus>,
    ) {
        if let Some(u) = username {
            self.username = u;
        }
        if let Some(g) = group_id {
            self.group_id = g;
        }
        if let Some(s) = status {
            self.status = s;
        }
        self.updated_at = Utc::now();
    }

    pub fn set_password_hash(&mut self, hash: String) {
        self.password_hash = hash;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_active_status_and_unique_ids() {
        let a = User::new("alice".into(), "h".into(), None, UserRole::Regular);
        let b = User::new("alice".into(), "h".into(), None, UserRole::Regular);
        assert_eq!(a.status, UserStatus::Active);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_stores_fields_correctly() {
        let gid = Uuid::new_v4();
        let u = User::new("bob".into(), "hash".into(), Some(gid), UserRole::Admin);
        assert_eq!(u.username, "bob");
        assert_eq!(u.password_hash, "hash");
        assert_eq!(u.group_id, Some(gid));
        assert_eq!(u.role, UserRole::Admin);
        assert_eq!(u.status, UserStatus::Active);
    }

    #[test]
    fn is_admin_returns_true_only_for_admin_role() {
        let admin = User::new("a".into(), "h".into(), None, UserRole::Admin);
        let regular = User::new("r".into(), "h".into(), None, UserRole::Regular);
        assert!(admin.is_admin());
        assert!(!regular.is_admin());
    }

    #[test]
    fn apply_update_username_only() {
        let mut u = User::new("old".into(), "h".into(), None, UserRole::Regular);
        let before = u.updated_at;
        u.apply_update(Some("new".into()), None, None);
        assert_eq!(u.username, "new");
        assert!(u.group_id.is_none());
        assert_eq!(u.status, UserStatus::Active);
        assert!(u.updated_at >= before);
    }

    #[test]
    fn apply_update_clears_group_id_with_some_none() {
        let gid = Uuid::new_v4();
        let mut u = User::new("u".into(), "h".into(), Some(gid), UserRole::Regular);
        u.apply_update(None, Some(None), None);
        assert!(u.group_id.is_none());
    }

    #[test]
    fn apply_update_sets_group_id_with_some_some() {
        let mut u = User::new("u".into(), "h".into(), None, UserRole::Regular);
        let gid = Uuid::new_v4();
        u.apply_update(None, Some(Some(gid)), None);
        assert_eq!(u.group_id, Some(gid));
    }

    #[test]
    fn apply_update_status() {
        let mut u = User::new("u".into(), "h".into(), None, UserRole::Regular);
        u.apply_update(None, None, Some(UserStatus::Inactive));
        assert_eq!(u.status, UserStatus::Inactive);
    }

    #[test]
    fn apply_update_none_args_leave_fields_unchanged() {
        let gid = Uuid::new_v4();
        let mut u = User::new("u".into(), "h".into(), Some(gid), UserRole::Admin);
        u.apply_update(None, None, None);
        assert_eq!(u.username, "u");
        assert_eq!(u.group_id, Some(gid));
        assert_eq!(u.status, UserStatus::Active);
    }

    #[test]
    fn set_password_hash_updates_hash_and_timestamp() {
        let mut u = User::new("u".into(), "old".into(), None, UserRole::Regular);
        let before = u.updated_at;
        u.set_password_hash("new".into());
        assert_eq!(u.password_hash, "new");
        assert!(u.updated_at >= before);
    }

    #[test]
    fn user_role_display_roundtrip() {
        for (role, s) in [(UserRole::Admin, "admin"), (UserRole::Regular, "regular")] {
            assert_eq!(role.to_string(), s);
            assert_eq!(s.parse::<UserRole>().unwrap(), role);
        }
    }

    #[test]
    fn user_role_from_str_unknown_returns_err() {
        assert!("".parse::<UserRole>().is_err());
        assert!("superuser".parse::<UserRole>().is_err());
    }

    #[test]
    fn user_status_display_roundtrip() {
        for (status, s) in [(UserStatus::Active, "active"), (UserStatus::Inactive, "inactive")] {
            assert_eq!(status.to_string(), s);
            assert_eq!(s.parse::<UserStatus>().unwrap(), status);
        }
    }

    #[test]
    fn user_status_from_str_unknown_returns_err() {
        assert!("".parse::<UserStatus>().is_err());
        assert!("banned".parse::<UserStatus>().is_err());
    }
}
