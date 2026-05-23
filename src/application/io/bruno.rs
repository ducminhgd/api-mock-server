use std::collections::HashMap;
use std::io::{Read, Write};

use uuid::Uuid;

use crate::application::io::{ImportedCollection, ImportedEndpoint};
use crate::domain::collection::Collection;
use crate::domain::endpoint::{Endpoint, HttpMethod};

// ── Block parsing ─────────────────────────────────────────────────────────────

struct Block {
    name: String,
    lines: Vec<String>,
}

fn parse_blocks(content: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current: Option<Block> = None;
    let mut depth: u32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(ref mut block) = current {
            if trimmed == "}" && depth == 1 {
                blocks.push(Block {
                    name: block.name.clone(),
                    lines: block.lines.clone(),
                });
                current = None;
                depth = 0;
            } else {
                // Track inner braces so body blocks with JSON don't end early
                depth += trimmed.chars().filter(|&c| c == '{').count() as u32;
                depth = depth.saturating_sub(trimmed.chars().filter(|&c| c == '}').count() as u32);
                block.lines.push(line.to_string());
            }
        } else if trimmed.ends_with('{') && !trimmed.starts_with('#') {
            let name = trimmed.trim_end_matches('{').trim().to_string();
            current = Some(Block {
                name,
                lines: Vec::new(),
            });
            depth = 1;
        }
    }

    blocks
}

fn block_kv(block: &Block) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in &block.lines {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find(':') {
            let key = trimmed[..pos].trim().to_string();
            let value = trimmed[pos + 1..].trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

// ── Import helpers ────────────────────────────────────────────────────────────

// Extracts path from a URL string, stripping the host/base-url variable.
fn extract_path_from_url(url: &str) -> String {
    let s = url.trim();

    let after_host = if let Some(pos) = s.find("://") {
        let rest = &s[pos + 3..];
        rest.find('/').map(|p| &rest[p..]).unwrap_or("/")
    } else if s.starts_with("{{") {
        // {{BASE_URL}}/path
        s.find('/').map(|p| &s[p..]).unwrap_or("/")
    } else {
        s
    };

    let path = after_host.split('?').next().unwrap_or("/");
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    // Strip /mocks/{uuid} prefix if present (re-imported from our own export)
    let path = strip_mocks_prefix(&path);

    // Convert :variable → {variable} and {{variable}} → {variable}
    convert_params(&path)
}

fn strip_mocks_prefix(path: &str) -> String {
    // /mocks/<uuid>/rest  →  /rest
    if let Some(rest) = path.strip_prefix("/mocks/") {
        // Try to strip a UUID segment
        let seg_end = rest.find('/').unwrap_or(rest.len());
        let candidate = &rest[..seg_end];
        if Uuid::parse_str(candidate).is_ok() {
            let remainder = &rest[seg_end..];
            return if remainder.is_empty() {
                "/".into()
            } else {
                remainder.to_string()
            };
        }
    }
    path.to_string()
}

fn convert_params(path: &str) -> String {
    // {{variable}} → {variable}
    let step1 = path.replace("{{", "{").replace("}}", "}");
    // :variable → {variable} in each segment
    step1
        .split('/')
        .map(|seg| {
            if let Some(name) = seg.strip_prefix(':') {
                format!("{{{name}}}")
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

// ── Parse single .bru file ────────────────────────────────────────────────────

pub fn parse_single_bru(collection_name: &str, content: &str) -> Option<ImportedCollection> {
    let ep = bru_to_endpoint(content)?;
    Some(ImportedCollection {
        name: collection_name.to_string(),
        description: None,
        endpoints: vec![ep],
    })
}

fn bru_to_endpoint(content: &str) -> Option<ImportedEndpoint> {
    let blocks = parse_blocks(content);

    let mut name = "Endpoint".to_string();
    let mut method: Option<HttpMethod> = None;
    let mut path = "/".to_string();

    for block in &blocks {
        match block.name.as_str() {
            "meta" => {
                let kv = block_kv(block);
                if let Some(n) = kv.get("name") {
                    name = n.clone();
                }
            }
            m @ ("get" | "post" | "put" | "patch" | "delete" | "head" | "options") => {
                method = m.to_uppercase().parse::<HttpMethod>().ok();
                let kv = block_kv(block);
                if let Some(url) = kv.get("url") {
                    path = extract_path_from_url(url);
                }
            }
            _ => {}
        }
    }

    Some(ImportedEndpoint {
        name,
        method: method?,
        path,
        status_code: 200,
        response_headers: None,
        response_body: None,
        response_content_type: None,
        delay_ms: 0,
    })
}

// ── Parse ZIP (Bruno collection export) ──────────────────────────────────────

pub fn parse_zip(data: &[u8]) -> Result<ImportedCollection, String> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    let mut collection_name = "Imported Collection".to_string();
    let mut collection_description: Option<String> = None;
    let mut endpoints: Vec<ImportedEndpoint> = Vec::new();

    // First pass: look for bruno.json for collection metadata
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let fname = file.name().to_string();
        if fname == "bruno.json" || fname.ends_with("/bruno.json") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| e.to_string())?;
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(n) = meta["name"].as_str() {
                    collection_name = n.to_string();
                }
                if let Some(d) = meta["description"].as_str() {
                    collection_description = Some(d.to_string());
                }
            }
        }
    }

    // Second pass: parse .bru files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let fname = file.name().to_string();
        if fname.ends_with(".bru") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| e.to_string())?;
            if let Some(ep) = bru_to_endpoint(&content) {
                endpoints.push(ep);
            }
        }
    }

    Ok(ImportedCollection {
        name: collection_name,
        description: collection_description,
        endpoints,
    })
}

// ── Serialize (export) ────────────────────────────────────────────────────────

// {variable} → {{variable}}
fn ours_to_bruno(path: &str) -> String {
    path.replace('{', "{{").replace('}', "}}")
}

fn endpoint_to_bru(collection_id: Uuid, ep: &Endpoint, seq: usize) -> String {
    let method_lower = ep.method.to_string().to_lowercase();
    let bruno_path = ours_to_bruno(&ep.path);
    let url = format!("{{{{BASE_URL}}}}/mocks/{collection_id}{bruno_path}");

    let mut bru = format!(
        "meta {{\n  name: {}\n  type: http\n  seq: {seq}\n}}\n\n",
        ep.name
    );

    bru.push_str(&format!(
        "{method_lower} {{\n  url: {url}\n  body: none\n  auth: none\n}}\n\n"
    ));

    // headers block (always written, even if empty)
    bru.push_str("headers {\n}\n\n");

    // docs block encodes mock response config
    let mut docs_lines = vec![format!("Mock status: {}", ep.status_code)];
    if let Some(ref ct) = ep.response_content_type {
        docs_lines.push(format!("Mock Content-Type: {ct}"));
    }
    if let Some(ref h) = ep.response_headers {
        docs_lines.push(format!("Mock headers: {h}"));
    }
    if let Some(ref body) = ep.response_body {
        docs_lines.push(String::new());
        docs_lines.push(body.clone());
    }

    bru.push_str("docs {\n");
    for line in &docs_lines {
        bru.push_str(&format!("  {line}\n"));
    }
    bru.push_str("}\n");

    bru
}

fn safe_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn serialize_zip(collection: &Collection, endpoints: &[Endpoint]) -> Result<Vec<u8>, String> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Write bruno.json metadata
    let meta = serde_json::json!({
        "version": "1",
        "name": collection.name,
        "description": collection.description,
        "type": "collection"
    });
    zip.start_file("bruno.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(meta.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    for (i, ep) in endpoints.iter().enumerate() {
        let bru_content = endpoint_to_bru(collection.id, ep, i + 1);
        let filename = format!("{}.bru", safe_filename(&ep.name));
        zip.start_file(&filename, options)
            .map_err(|e| e.to_string())?;
        zip.write_all(bru_content.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_bru(name: &str, method: &str, url: &str) -> String {
        format!(
            "meta {{\n  name: {name}\n  type: http\n  seq: 1\n}}\n\n\
             {method} {{\n  url: {url}\n  body: none\n}}\n\n\
             headers {{\n}}\n"
        )
    }

    #[test]
    fn bru_to_endpoint_extracts_name_and_method() {
        let bru = sample_bru("List Users", "get", "{{BASE_URL}}/users");
        let ep = bru_to_endpoint(&bru).unwrap();
        assert_eq!(ep.name, "List Users");
        assert_eq!(ep.method, HttpMethod::Get);
    }

    #[test]
    fn bru_to_endpoint_extracts_path() {
        let bru = sample_bru("Get User", "get", "{{BASE_URL}}/users/{{id}}");
        let ep = bru_to_endpoint(&bru).unwrap();
        assert_eq!(ep.path, "/users/{id}");
    }

    #[test]
    fn bru_to_endpoint_converts_colon_params() {
        let bru = sample_bru("Get User", "get", "{{BASE_URL}}/users/:id");
        let ep = bru_to_endpoint(&bru).unwrap();
        assert_eq!(ep.path, "/users/{id}");
    }

    #[test]
    fn bru_to_endpoint_strips_mocks_prefix() {
        let cid = Uuid::new_v4();
        let bru = sample_bru(
            "Get User",
            "get",
            &format!("{{{{BASE_URL}}}}/mocks/{cid}/users/{{{{id}}}}"),
        );
        let ep = bru_to_endpoint(&bru).unwrap();
        assert_eq!(ep.path, "/users/{id}");
    }

    #[test]
    fn bru_to_endpoint_returns_none_without_method_block() {
        let bru = "meta {\n  name: Test\n  type: http\n}\n\nheaders {\n}\n";
        assert!(bru_to_endpoint(bru).is_none());
    }

    #[test]
    fn bru_to_endpoint_post_method() {
        let bru = sample_bru("Create User", "post", "{{BASE_URL}}/users");
        let ep = bru_to_endpoint(&bru).unwrap();
        assert_eq!(ep.method, HttpMethod::Post);
    }

    #[test]
    fn parse_single_bru_wraps_into_collection() {
        let bru = sample_bru("Ping", "get", "{{BASE_URL}}/ping");
        let c = parse_single_bru("My API", &bru).unwrap();
        assert_eq!(c.name, "My API");
        assert_eq!(c.endpoints.len(), 1);
    }

    #[test]
    fn serialize_zip_produces_non_empty_bytes() {
        use crate::domain::collection::{Collection, CollectionVisibility};
        let owner = Uuid::new_v4();
        let c = Collection::new("Test".into(), None, owner, CollectionVisibility::Private);
        let ep = Endpoint::new(
            c.id,
            "List".into(),
            HttpMethod::Get,
            "/items".into(),
            200,
            0,
            None,
            Some("[]".into()),
            Some("application/json".into()),
        );
        let bytes = serialize_zip(&c, &[ep]).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn serialize_and_parse_zip_roundtrip() {
        use crate::domain::collection::{Collection, CollectionVisibility};
        let owner = Uuid::new_v4();
        let c = Collection::new(
            "Round Trip".into(),
            None,
            owner,
            CollectionVisibility::Private,
        );
        let ep = Endpoint::new(
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
        let bytes = serialize_zip(&c, &[ep]).unwrap();

        let imported = parse_zip(&bytes).unwrap();
        assert_eq!(imported.name, "Round Trip");
        assert_eq!(imported.endpoints.len(), 1);
        assert_eq!(imported.endpoints[0].method, HttpMethod::Get);
        assert_eq!(imported.endpoints[0].path, "/users");
    }

    #[test]
    fn extract_path_from_url_full_url() {
        assert_eq!(
            extract_path_from_url("https://api.example.com/v1/users"),
            "/v1/users"
        );
    }

    #[test]
    fn extract_path_from_url_drops_query() {
        assert_eq!(extract_path_from_url("{{BASE_URL}}/users?page=1"), "/users");
    }

    #[test]
    fn convert_params_handles_both_styles() {
        assert_eq!(convert_params("/users/{{id}}"), "/users/{id}");
        assert_eq!(convert_params("/users/:id"), "/users/{id}");
    }

    #[test]
    fn ours_to_bruno_converts_single_braces() {
        assert_eq!(ours_to_bruno("/users/{id}"), "/users/{{id}}");
    }
}
