use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            other => Err(format!("unknown HTTP method: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for EndpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointStatus::Active => write!(f, "active"),
            EndpointStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl std::str::FromStr for EndpointStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(EndpointStatus::Active),
            "inactive" => Ok(EndpointStatus::Inactive),
            other => Err(format!("unknown endpoint status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub status_code: u16,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_content_type: Option<String>,
    pub delay_ms: u32,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Endpoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        collection_id: Uuid,
        name: String,
        method: HttpMethod,
        path: String,
        status_code: u16,
        delay_ms: u32,
        response_headers: Option<String>,
        response_body: Option<String>,
        response_content_type: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            collection_id,
            name,
            method,
            path,
            status_code,
            response_headers,
            response_body,
            response_content_type,
            delay_ms,
            status: EndpointStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_update(
        &mut self,
        name: Option<String>,
        method: Option<HttpMethod>,
        path: Option<String>,
        status_code: Option<u16>,
        delay_ms: Option<u32>,
        response_headers: Option<Option<String>>,
        response_body: Option<Option<String>>,
        response_content_type: Option<Option<String>>,
        status: Option<EndpointStatus>,
    ) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(m) = method {
            self.method = m;
        }
        if let Some(p) = path {
            self.path = p;
        }
        if let Some(sc) = status_code {
            self.status_code = sc;
        }
        if let Some(d) = delay_ms {
            self.delay_ms = d;
        }
        if let Some(rh) = response_headers {
            self.response_headers = rh;
        }
        if let Some(rb) = response_body {
            self.response_body = rb;
        }
        if let Some(rct) = response_content_type {
            self.response_content_type = rct;
        }
        if let Some(s) = status {
            self.status = s;
        }
        self.updated_at = Utc::now();
    }

    pub fn copy_to(&self, new_collection_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            collection_id: new_collection_id,
            name: self.name.clone(),
            method: self.method.clone(),
            path: self.path.clone(),
            status_code: self.status_code,
            response_headers: self.response_headers.clone(),
            response_body: self.response_body.clone(),
            response_content_type: self.response_content_type.clone(),
            delay_ms: self.delay_ms,
            status: EndpointStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid() -> Uuid {
        Uuid::new_v4()
    }

    fn make_endpoint(name: &str) -> Endpoint {
        Endpoint::new(
            cid(),
            name.into(),
            HttpMethod::Get,
            "/test".into(),
            200,
            0,
            None,
            None,
            None,
        )
    }

    #[test]
    fn new_sets_active_status() {
        let e = make_endpoint("EP");
        assert_eq!(e.status, EndpointStatus::Active);
    }

    #[test]
    fn new_defaults_are_set() {
        let e = make_endpoint("EP");
        assert_eq!(e.status_code, 200);
        assert_eq!(e.delay_ms, 0);
        assert!(e.response_headers.is_none());
        assert!(e.response_body.is_none());
        assert!(e.response_content_type.is_none());
    }

    #[test]
    fn new_assigns_unique_ids() {
        let a = make_endpoint("A");
        let b = make_endpoint("B");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn apply_update_name_only() {
        let mut e = make_endpoint("Old");
        let before = e.updated_at;
        e.apply_update(
            Some("New".into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(e.name, "New");
        assert_eq!(e.method, HttpMethod::Get);
        assert!(e.updated_at >= before);
    }

    #[test]
    fn apply_update_method() {
        let mut e = make_endpoint("E");
        e.apply_update(
            None,
            Some(HttpMethod::Post),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(e.method, HttpMethod::Post);
    }

    #[test]
    fn apply_update_status_code_and_delay() {
        let mut e = make_endpoint("E");
        e.apply_update(
            None,
            None,
            None,
            Some(404),
            Some(100),
            None,
            None,
            None,
            None,
        );
        assert_eq!(e.status_code, 404);
        assert_eq!(e.delay_ms, 100);
    }

    #[test]
    fn apply_update_clears_response_body_with_some_none() {
        let mut e = Endpoint::new(
            cid(),
            "E".into(),
            HttpMethod::Get,
            "/".into(),
            200,
            0,
            None,
            Some("body".into()),
            None,
        );
        e.apply_update(None, None, None, None, None, None, Some(None), None, None);
        assert!(e.response_body.is_none());
    }

    #[test]
    fn apply_update_sets_response_headers() {
        let mut e = make_endpoint("E");
        e.apply_update(
            None,
            None,
            None,
            None,
            None,
            Some(Some("{\"X-Foo\":\"bar\"}".into())),
            None,
            None,
            None,
        );
        assert_eq!(e.response_headers.as_deref(), Some("{\"X-Foo\":\"bar\"}"));
    }

    #[test]
    fn apply_update_status_inactive() {
        let mut e = make_endpoint("E");
        e.apply_update(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(EndpointStatus::Inactive),
        );
        assert_eq!(e.status, EndpointStatus::Inactive);
    }

    #[test]
    fn copy_to_creates_new_id_and_collection_id() {
        let e = make_endpoint("Original");
        let new_cid = Uuid::new_v4();
        let copy = e.copy_to(new_cid);
        assert_ne!(copy.id, e.id);
        assert_eq!(copy.collection_id, new_cid);
        assert_eq!(copy.name, "Original");
        assert_eq!(copy.method, HttpMethod::Get);
        assert_eq!(copy.status, EndpointStatus::Active);
    }

    #[test]
    fn http_method_display_roundtrip() {
        for (method, s) in [
            (HttpMethod::Get, "GET"),
            (HttpMethod::Post, "POST"),
            (HttpMethod::Put, "PUT"),
            (HttpMethod::Patch, "PATCH"),
            (HttpMethod::Delete, "DELETE"),
            (HttpMethod::Head, "HEAD"),
            (HttpMethod::Options, "OPTIONS"),
        ] {
            assert_eq!(method.to_string(), s);
            assert_eq!(s.parse::<HttpMethod>().unwrap(), method);
        }
    }

    #[test]
    fn http_method_from_str_case_insensitive() {
        assert_eq!("get".parse::<HttpMethod>().unwrap(), HttpMethod::Get);
        assert_eq!("Post".parse::<HttpMethod>().unwrap(), HttpMethod::Post);
    }

    #[test]
    fn http_method_from_str_unknown_returns_err() {
        assert!("".parse::<HttpMethod>().is_err());
        assert!("TRACE".parse::<HttpMethod>().is_err());
    }

    #[test]
    fn endpoint_status_display_roundtrip() {
        for (status, s) in [
            (EndpointStatus::Active, "active"),
            (EndpointStatus::Inactive, "inactive"),
        ] {
            assert_eq!(status.to_string(), s);
            assert_eq!(s.parse::<EndpointStatus>().unwrap(), status);
        }
    }

    #[test]
    fn endpoint_status_from_str_unknown_returns_err() {
        assert!("".parse::<EndpointStatus>().is_err());
        assert!("disabled".parse::<EndpointStatus>().is_err());
    }
}
