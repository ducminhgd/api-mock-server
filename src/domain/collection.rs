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

/// Derives a Jira-style project key from a collection name.
/// Multi-word names produce an acronym (first letter of each word).
/// Single-word names use the first 10 characters.
/// Result is uppercase, letters and digits only, starts with a letter, max 10 chars.
pub fn slugify_code(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_uppercase())
        .collect();

    let raw: String = if words.len() > 1 {
        words.iter().filter_map(|w| w.chars().next()).collect()
    } else if let Some(word) = words.first() {
        word.chars()
            .skip_while(|c| c.is_ascii_digit())
            .take(3)
            .collect()
    } else {
        String::new()
    };

    // Keep only ASCII letters and digits; strip any leading digits so it starts with a letter.
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        .skip_while(|c| c.is_ascii_digit())
        .collect();

    if cleaned.is_empty() {
        "C".to_string()
    } else {
        cleaned.chars().take(10).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub code: String,
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
        code: String,
        description: Option<String>,
        owner_id: Uuid,
        visibility: CollectionVisibility,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            code,
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
        code: Option<String>,
        description: Option<Option<String>>,
        status: Option<CollectionStatus>,
        visibility: Option<CollectionVisibility>,
    ) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(c) = code {
            self.code = c;
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
        let c = Collection::new(
            "My API".into(),
            "my-api".into(),
            None,
            owner(),
            CollectionVisibility::Private,
        );
        assert_eq!(c.status, CollectionStatus::Active);
        assert_eq!(c.visibility, CollectionVisibility::Private);
        assert!(c.description.is_none());
    }

    #[test]
    fn new_assigns_unique_ids() {
        let oid = owner();
        let a = Collection::new(
            "A".into(),
            "a".into(),
            None,
            oid,
            CollectionVisibility::Private,
        );
        let b = Collection::new(
            "A".into(),
            "a".into(),
            None,
            oid,
            CollectionVisibility::Private,
        );
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn new_stores_public_visibility() {
        let c = Collection::new(
            "Pub".into(),
            "pub".into(),
            None,
            owner(),
            CollectionVisibility::Public,
        );
        assert_eq!(c.visibility, CollectionVisibility::Public);
    }

    #[test]
    fn new_stores_description() {
        let c = Collection::new(
            "C".into(),
            "c".into(),
            Some("desc".into()),
            owner(),
            CollectionVisibility::Private,
        );
        assert_eq!(c.description.as_deref(), Some("desc"));
    }

    #[test]
    fn apply_update_name_only() {
        let mut c = Collection::new(
            "Old".into(),
            "old".into(),
            None,
            owner(),
            CollectionVisibility::Private,
        );
        let before = c.updated_at;
        c.apply_update(Some("New".into()), None, None, None, None);
        assert_eq!(c.name, "New");
        assert!(c.description.is_none());
        assert_eq!(c.status, CollectionStatus::Active);
        assert_eq!(c.visibility, CollectionVisibility::Private);
        assert!(c.updated_at >= before);
    }

    #[test]
    fn apply_update_clears_description_with_some_none() {
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            Some("desc".into()),
            owner(),
            CollectionVisibility::Private,
        );
        c.apply_update(None, None, Some(None), None, None);
        assert!(c.description.is_none());
    }

    #[test]
    fn apply_update_sets_description_with_some_some() {
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            None,
            owner(),
            CollectionVisibility::Private,
        );
        c.apply_update(None, None, Some(Some("new desc".into())), None, None);
        assert_eq!(c.description.as_deref(), Some("new desc"));
    }

    #[test]
    fn apply_update_status() {
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            None,
            owner(),
            CollectionVisibility::Private,
        );
        c.apply_update(None, None, None, Some(CollectionStatus::Inactive), None);
        assert_eq!(c.status, CollectionStatus::Inactive);
    }

    #[test]
    fn apply_update_visibility() {
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            None,
            owner(),
            CollectionVisibility::Private,
        );
        c.apply_update(None, None, None, None, Some(CollectionVisibility::Public));
        assert_eq!(c.visibility, CollectionVisibility::Public);
    }

    #[test]
    fn apply_update_none_args_leave_fields_unchanged() {
        let oid = owner();
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            Some("d".into()),
            oid,
            CollectionVisibility::Public,
        );
        c.apply_update(None, None, None, None, None);
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
        let mut c = Collection::new(
            "C".into(),
            "c".into(),
            None,
            original_owner,
            CollectionVisibility::Private,
        );
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

    #[test]
    fn slugify_code_multi_word_produces_acronym() {
        assert_eq!(slugify_code("My API"), "MA");
        assert_eq!(slugify_code("Hello World!"), "HW");
        assert_eq!(slugify_code("API Mock Server"), "AMS");
        assert_eq!(slugify_code("my-api"), "MA");
    }

    #[test]
    fn slugify_code_single_word_takes_up_to_3_chars() {
        assert_eq!(slugify_code("abc"), "ABC");
        assert_eq!(slugify_code("development"), "DEV");
        assert_eq!(slugify_code("AB"), "AB");
        assert_eq!(slugify_code("A"), "A");
    }

    #[test]
    fn slugify_code_blank_returns_fallback() {
        assert_eq!(slugify_code("  "), "C");
        assert_eq!(slugify_code("!!!"), "C");
    }

    #[test]
    fn slugify_code_strips_leading_digits() {
        assert_eq!(slugify_code("123project"), "PRO");
        assert_eq!(slugify_code("1"), "C");
    }

    #[test]
    fn slugify_code_single_word_truncates_at_3_chars() {
        let long = "a".repeat(40);
        assert_eq!(slugify_code(&long).len(), 3);
    }

    #[test]
    fn slugify_code_multi_word_acronym_truncates_at_10_chars() {
        // 11 single-char words → acronym of 11 chars → truncated to 10
        let long_name = "a b c d e f g h i j k";
        assert_eq!(slugify_code(long_name).len(), 10);
    }
}
