use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::io::{ImportedCollection, ImportedEndpoint};
use crate::domain::collection::Collection;
use crate::domain::endpoint::{Endpoint, HttpMethod};

// ── Postman Collection v2.1 wire types ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    #[serde(default)]
    pub item: Vec<PostmanItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanInfo {
    #[serde(rename = "_postman_id", skip_serializing_if = "Option::is_none")]
    pub postman_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<PostmanDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanDescription {
    Text(String),
    Object {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },
}

impl PostmanDescription {
    fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            Self::Object { content } => content.as_deref(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<PostmanRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response: Vec<PostmanResponse>,
    // Folder: items nested under a folder
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item: Vec<PostmanItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<PostmanUrl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub header: Vec<PostmanHeader>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanUrl {
    String(String),
    Object(PostmanUrlObject),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanUrlObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<PostmanPathSegment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
}

// Path segments may be a plain string or an object with a `value` field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanPathSegment {
    String(String),
    Object { value: Option<String> },
}

impl PostmanPathSegment {
    fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s.as_str(),
            Self::Object { value: Some(v) } => v.as_str(),
            Self::Object { value: None } => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanHeader {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostmanResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub header: Vec<PostmanHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(
        rename = "_postman_previewlanguage",
        skip_serializing_if = "Option::is_none"
    )]
    pub preview_language: Option<String>,
}

// ── Parse (import) ────────────────────────────────────────────────────────────

pub fn parse(json: &str) -> Result<ImportedCollection, serde_json::Error> {
    let raw: PostmanCollection = serde_json::from_str(json)?;

    let name = raw.info.name.clone();
    let description = raw
        .info
        .description
        .as_ref()
        .and_then(|d| d.as_text())
        .map(String::from);

    let mut endpoints = Vec::new();
    flatten_items(&raw.item, &mut endpoints);

    Ok(ImportedCollection {
        name,
        description,
        endpoints,
    })
}

fn flatten_items(items: &[PostmanItem], out: &mut Vec<ImportedEndpoint>) {
    for item in items {
        if !item.item.is_empty() {
            // Folder — recurse
            flatten_items(&item.item, out);
        } else if let Some(req) = &item.request {
            let name = item.name.clone().unwrap_or_else(|| "Endpoint".into());
            let method = req
                .method
                .as_deref()
                .and_then(|m| m.parse::<HttpMethod>().ok())
                .unwrap_or(HttpMethod::Get);
            let path = req
                .url
                .as_ref()
                .map(extract_path)
                .unwrap_or_else(|| "/".into());

            // Use the first saved response for mock config, fall back to defaults.
            let first_resp = item.response.first();
            let status_code = first_resp.and_then(|r| r.code).unwrap_or(200);
            let response_body = first_resp.and_then(|r| r.body.clone());

            let (response_content_type, response_headers) = first_resp
                .map(|r| extract_headers(&r.header))
                .unwrap_or((None, None));

            out.push(ImportedEndpoint {
                name,
                method,
                path,
                status_code,
                response_headers,
                response_body,
                response_content_type,
                delay_ms: 0,
            });
        }
    }
}

// Returns (content_type, other_headers_json)
fn extract_headers(headers: &[PostmanHeader]) -> (Option<String>, Option<String>) {
    let mut ct: Option<String> = None;
    let mut map: HashMap<String, String> = HashMap::new();

    for h in headers {
        if h.disabled == Some(true) {
            continue;
        }
        if h.key.to_lowercase() == "content-type" {
            ct = Some(h.value.clone());
        } else {
            map.insert(h.key.clone(), h.value.clone());
        }
    }

    let headers_json = if map.is_empty() {
        None
    } else {
        serde_json::to_string(&map).ok()
    };

    (ct, headers_json)
}

fn extract_path(url: &PostmanUrl) -> String {
    match url {
        PostmanUrl::String(s) => path_from_raw_url(s),
        PostmanUrl::Object(obj) => {
            if !obj.path.is_empty() {
                let joined = obj
                    .path
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("/");
                let p = format!("/{joined}");
                postman_params_to_ours(&p)
            } else if let Some(raw) = &obj.raw {
                path_from_raw_url(raw)
            } else {
                "/".into()
            }
        }
    }
}

fn path_from_raw_url(raw: &str) -> String {
    let s = raw.trim();

    // Strip scheme://host
    let after_host = if let Some(pos) = s.find("://") {
        let rest = &s[pos + 3..];
        rest.find('/').map(|p| &rest[p..]).unwrap_or("/")
    } else if s.starts_with("{{") || s.starts_with(':') {
        // {{base_url}}/path  or  :host/path
        s.find('/').map(|p| &s[p..]).unwrap_or("/")
    } else {
        s
    };

    // Drop query string
    let path = after_host.split('?').next().unwrap_or("/");
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    postman_params_to_ours(&path)
}

// {{variable}} → {variable}
fn postman_params_to_ours(path: &str) -> String {
    path.replace("{{", "{").replace("}}", "}")
}

// {variable} → {{variable}}
fn ours_to_postman_params(path: &str) -> String {
    path.replace('{', "{{").replace('}', "}}")
}

// ── Serialize (export) ────────────────────────────────────────────────────────

pub fn serialize(
    collection: &Collection,
    endpoints: &[Endpoint],
) -> Result<String, serde_json::Error> {
    let items: Vec<PostmanItem> = endpoints
        .iter()
        .map(|ep| endpoint_to_item(collection.id, ep))
        .collect();

    let pc = PostmanCollection {
        info: PostmanInfo {
            postman_id: Some(collection.id.to_string()),
            name: collection.name.clone(),
            description: collection
                .description
                .as_ref()
                .map(|d| PostmanDescription::Text(d.clone())),
            schema: Some(
                "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(),
            ),
        },
        item: items,
    };

    serde_json::to_string_pretty(&pc)
}

fn endpoint_to_item(collection_id: Uuid, ep: &Endpoint) -> PostmanItem {
    let postman_path = ours_to_postman_params(&ep.path);
    let raw_url = format!("{{{{base_url}}}}/mocks/{collection_id}{postman_path}");

    // Build path segments for the URL object (strip leading /)
    let path_segs: Vec<PostmanPathSegment> = {
        let mut segs = vec![
            PostmanPathSegment::String("mocks".into()),
            PostmanPathSegment::String(collection_id.to_string()),
        ];
        for seg in ep.path.trim_matches('/').split('/') {
            if !seg.is_empty() {
                segs.push(PostmanPathSegment::String(ours_to_postman_params(seg)));
            }
        }
        segs
    };

    let request = PostmanRequest {
        method: Some(ep.method.to_string()),
        url: Some(PostmanUrl::Object(PostmanUrlObject {
            raw: Some(raw_url),
            path: path_segs,
            host: vec!["{{base_url}}".into()],
            protocol: Some("https".into()),
        })),
        header: vec![],
    };

    // Build saved response headers
    let mut resp_headers: Vec<PostmanHeader> = Vec::new();
    if let Some(ref ct) = ep.response_content_type {
        resp_headers.push(PostmanHeader {
            key: "Content-Type".into(),
            value: ct.clone(),
            disabled: None,
        });
    }
    if let Some(ref h_json) = ep.response_headers {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(h_json) {
            for (k, v) in map {
                resp_headers.push(PostmanHeader {
                    key: k,
                    value: v,
                    disabled: None,
                });
            }
        }
    }

    let saved_response = PostmanResponse {
        name: Some("Mock Response".into()),
        status: Some(status_text(ep.status_code)),
        code: Some(ep.status_code),
        header: resp_headers,
        body: ep.response_body.clone(),
        preview_language: ep.response_content_type.as_ref().map(|ct| {
            if ct.contains("json") {
                "json".into()
            } else if ct.contains("xml") {
                "xml".into()
            } else if ct.contains("html") {
                "html".into()
            } else {
                "text".into()
            }
        }),
    };

    PostmanItem {
        name: Some(ep.name.clone()),
        request: Some(request),
        response: vec![saved_response],
        item: vec![],
    }
}

fn status_text(code: u16) -> String {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
    .into()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn postman_json(
        name: &str,
        method: &str,
        path_segs: &[&str],
        resp_code: u16,
        resp_body: &str,
    ) -> String {
        let segs: Vec<String> = path_segs.iter().map(|s| format!("\"{}\"", s)).collect();
        let segs_json = segs.join(", ");
        format!(
            r#"{{
  "info": {{ "name": "{name}", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json" }},
  "item": [
    {{
      "name": "ep1",
      "request": {{
        "method": "{method}",
        "url": {{
          "raw": "{{{{base_url}}}}/{path_segs_raw}",
          "path": [{segs_json}]
        }},
        "header": []
      }},
      "response": [
        {{
          "name": "ok",
          "code": {resp_code},
          "header": [{{"key": "Content-Type", "value": "application/json"}}],
          "body": "{resp_body}"
        }}
      ]
    }}
  ]
}}"#,
            path_segs_raw = path_segs.join("/")
        )
    }

    #[test]
    fn parse_extracts_collection_name() {
        let json = postman_json("My API", "GET", &["users"], 200, "[]");
        let result = parse(&json).unwrap();
        assert_eq!(result.name, "My API");
    }

    #[test]
    fn parse_extracts_endpoint_method_and_path() {
        let json = postman_json("API", "POST", &["users"], 201, "{}");
        let result = parse(&json).unwrap();
        assert_eq!(result.endpoints.len(), 1);
        let ep = &result.endpoints[0];
        assert_eq!(ep.method, HttpMethod::Post);
        assert_eq!(ep.path, "/users");
    }

    #[test]
    fn parse_extracts_status_code_and_body_from_first_response() {
        let json = postman_json("API", "GET", &["users"], 200, "ok");
        let result = parse(&json).unwrap();
        let ep = &result.endpoints[0];
        assert_eq!(ep.status_code, 200);
        assert_eq!(ep.response_body.as_deref(), Some("ok"));
    }

    #[test]
    fn parse_extracts_content_type_from_response_headers() {
        let json = postman_json("API", "GET", &["users"], 200, "[]");
        let result = parse(&json).unwrap();
        let ep = &result.endpoints[0];
        assert_eq!(
            ep.response_content_type.as_deref(),
            Some("application/json")
        );
    }

    #[test]
    fn parse_converts_postman_path_params_to_our_format() {
        let json = postman_json("API", "GET", &["users", "{{id}}"], 200, "{}");
        let result = parse(&json).unwrap();
        let ep = &result.endpoints[0];
        assert_eq!(ep.path, "/users/{id}");
    }

    #[test]
    fn parse_url_as_string_extracts_path() {
        let json = r#"{
            "info": {"name": "X", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
            "item": [{
                "name": "ep",
                "request": {"method": "GET", "url": "https://{{base_url}}/api/v1/users"},
                "response": []
            }]
        }"#;
        let result = parse(json).unwrap();
        assert_eq!(result.endpoints[0].path, "/api/v1/users");
    }

    #[test]
    fn parse_defaults_to_200_when_no_saved_response() {
        let json = r#"{
            "info": {"name": "X", "schema": "..."},
            "item": [{
                "name": "ep",
                "request": {"method": "DELETE", "url": {"raw": "/items/1", "path": ["items", "1"]}},
                "response": []
            }]
        }"#;
        let result = parse(json).unwrap();
        assert_eq!(result.endpoints[0].status_code, 200);
    }

    #[test]
    fn parse_flattens_nested_folders() {
        let json = r#"{
            "info": {"name": "X", "schema": "..."},
            "item": [{
                "name": "folder",
                "item": [{
                    "name": "inner",
                    "request": {"method": "GET", "url": {"raw": "/ping", "path": ["ping"]}},
                    "response": []
                }]
            }]
        }"#;
        let result = parse(json).unwrap();
        assert_eq!(result.endpoints.len(), 1);
        assert_eq!(result.endpoints[0].path, "/ping");
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        assert!(parse("{not json}").is_err());
    }

    #[test]
    fn postman_params_conversion_roundtrip() {
        assert_eq!(
            postman_params_to_ours("/users/{{id}}/posts/{{postId}}"),
            "/users/{id}/posts/{postId}"
        );
        assert_eq!(
            ours_to_postman_params("/users/{id}/posts/{postId}"),
            "/users/{{id}}/posts/{{postId}}"
        );
    }

    #[test]
    fn serialize_produces_valid_json_with_correct_name() {
        use crate::domain::collection::{Collection, CollectionVisibility};
        use uuid::Uuid;

        let owner = Uuid::new_v4();
        let c = Collection::new(
            "Test API".into(),
            None,
            owner,
            CollectionVisibility::Private,
        );
        let ep = crate::domain::endpoint::Endpoint::new(
            c.id,
            "Get Users".into(),
            HttpMethod::Get,
            "/users".into(),
            200,
            0,
            None,
            Some("[{}]".into()),
            Some("application/json".into()),
        );

        let json = serialize(&c, &[ep]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["info"]["name"], "Test API");
        assert_eq!(parsed["item"][0]["name"], "Get Users");
        assert_eq!(parsed["item"][0]["request"]["method"], "GET");
    }
}
