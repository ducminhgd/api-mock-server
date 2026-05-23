use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for GroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupStatus::Active => write!(f, "active"),
            GroupStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl std::str::FromStr for GroupStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(GroupStatus::Active),
            "inactive" => Ok(GroupStatus::Inactive),
            other => Err(format!("unknown group status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: GroupStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: GroupStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        status: Option<GroupStatus>,
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
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_active_status_and_unique_ids() {
        let a = Group::new("Engineering".into(), None);
        let b = Group::new("Engineering".into(), None);
        assert_eq!(a.status, GroupStatus::Active);
        assert_ne!(a.id, b.id);
        assert!(a.description.is_none());
    }

    #[test]
    fn new_stores_description() {
        let g = Group::new("Ops".into(), Some("Operations team".into()));
        assert_eq!(g.description.as_deref(), Some("Operations team"));
    }

    #[test]
    fn apply_update_name_only() {
        let mut g = Group::new("Old".into(), None);
        let before = g.updated_at;
        g.apply_update(Some("New".into()), None, None);
        assert_eq!(g.name, "New");
        assert!(g.description.is_none());
        assert_eq!(g.status, GroupStatus::Active);
        assert!(g.updated_at >= before);
    }

    #[test]
    fn apply_update_clears_description_with_some_none() {
        let mut g = Group::new("Ops".into(), Some("desc".into()));
        g.apply_update(None, Some(None), None);
        assert!(g.description.is_none());
    }

    #[test]
    fn apply_update_sets_description_with_some_some() {
        let mut g = Group::new("Ops".into(), None);
        g.apply_update(None, Some(Some("new desc".into())), None);
        assert_eq!(g.description.as_deref(), Some("new desc"));
    }

    #[test]
    fn apply_update_none_args_leave_fields_unchanged() {
        let mut g = Group::new("Ops".into(), Some("d".into()));
        g.apply_update(None, None, None);
        assert_eq!(g.name, "Ops");
        assert_eq!(g.description.as_deref(), Some("d"));
        assert_eq!(g.status, GroupStatus::Active);
    }

    #[test]
    fn apply_update_status() {
        let mut g = Group::new("Ops".into(), None);
        g.apply_update(None, None, Some(GroupStatus::Inactive));
        assert_eq!(g.status, GroupStatus::Inactive);
    }

    #[test]
    fn group_status_display_roundtrip() {
        for (status, s) in [
            (GroupStatus::Active, "active"),
            (GroupStatus::Inactive, "inactive"),
        ] {
            assert_eq!(status.to_string(), s);
            assert_eq!(s.parse::<GroupStatus>().unwrap(), status);
        }
    }

    #[test]
    fn group_status_from_str_unknown_returns_err() {
        assert!("unknown".parse::<GroupStatus>().is_err());
        assert!("".parse::<GroupStatus>().is_err());
    }
}
