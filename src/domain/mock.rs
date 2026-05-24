use crate::domain::endpoint::{Endpoint, HttpMethod};

/// Tries to match a stored path pattern against an incoming request path.
///
/// Pattern segments of the form `{anything}` are wildcards that match any
/// non-empty segment. Returns `Some(exact_count)` on a match, where
/// `exact_count` is the number of literal (non-wildcard) segments matched —
/// used to resolve priority when multiple patterns match.
pub fn match_path(pattern: &str, request_path: &str) -> Option<usize> {
    let pattern_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = request_path.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_segs.len() != path_segs.len() {
        return None;
    }

    let mut exact_count = 0usize;
    for (p, r) in pattern_segs.iter().zip(path_segs.iter()) {
        if p.starts_with('{') && p.ends_with('}') {
            // wildcard — matches any non-empty segment
        } else if p == r {
            exact_count += 1;
        } else {
            return None;
        }
    }

    Some(exact_count)
}

pub enum MockResolution<'a> {
    Matched(&'a Endpoint),
    NotFound,
    MethodNotAllowed(Vec<HttpMethod>),
}

/// Resolves an incoming (method, path) pair against a list of active endpoints.
///
/// Priority: among all path-matching endpoints, the one with the highest
/// number of literal (non-wildcard) segments wins. If no endpoint's path
/// matches, returns `NotFound`. If paths match but no method matches, returns
/// `MethodNotAllowed` with the list of supported methods.
pub fn resolve_endpoint<'a>(
    endpoints: &'a [Endpoint],
    method: &HttpMethod,
    path: &str,
) -> MockResolution<'a> {
    let path_matches: Vec<(&Endpoint, usize)> = endpoints
        .iter()
        .filter_map(|ep| match_path(&ep.path, path).map(|score| (ep, score)))
        .collect();

    if path_matches.is_empty() {
        return MockResolution::NotFound;
    }

    let method_matches: Vec<(&Endpoint, usize)> = path_matches
        .iter()
        .filter(|(ep, _)| &ep.method == method)
        .cloned()
        .collect();

    if method_matches.is_empty() {
        let allowed = path_matches
            .into_iter()
            .map(|(ep, _)| ep.method.clone())
            .collect();
        return MockResolution::MethodNotAllowed(allowed);
    }

    let best = method_matches
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(ep, _)| ep)
        .unwrap();

    MockResolution::Matched(best)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::domain::endpoint::{Endpoint, HttpMethod};

    fn ep(method: HttpMethod, path: &str) -> Endpoint {
        Endpoint::new(
            Uuid::new_v4(),
            "test".into(),
            method,
            path.into(),
            200,
            0,
            None,
            None,
            None,
        )
    }

    // ── match_path ────────────────────────────────────────────────────────────

    #[test]
    fn match_path_exact_returns_full_score() {
        assert_eq!(match_path("/users/me", "/users/me"), Some(2));
    }

    #[test]
    fn match_path_wildcard_segment_returns_partial_score() {
        assert_eq!(match_path("/users/{id}", "/users/123"), Some(1));
    }

    #[test]
    fn match_path_all_wildcards_returns_zero_score() {
        assert_eq!(match_path("/{a}/{b}", "/x/y"), Some(0));
    }

    #[test]
    fn match_path_literal_mismatch_returns_none() {
        assert_eq!(match_path("/users/me", "/users/you"), None);
    }

    #[test]
    fn match_path_segment_count_mismatch_returns_none() {
        assert_eq!(match_path("/users/{id}", "/users/1/orders"), None);
    }

    #[test]
    fn match_path_root_pattern_matches_root() {
        assert_eq!(match_path("/", "/"), Some(0));
    }

    #[test]
    fn match_path_root_does_not_match_subpath() {
        assert_eq!(match_path("/", "/users"), None);
    }

    #[test]
    fn match_path_normalises_leading_trailing_slashes() {
        assert_eq!(match_path("users/me", "/users/me"), Some(2));
        assert_eq!(match_path("/users/me/", "/users/me"), Some(2));
    }

    // ── resolve_endpoint ──────────────────────────────────────────────────────

    #[test]
    fn resolve_returns_matched_for_exact_hit() {
        let endpoints = vec![ep(HttpMethod::Get, "/users/me")];
        assert!(matches!(
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users/me"),
            MockResolution::Matched(_)
        ));
    }

    #[test]
    fn resolve_returns_not_found_when_no_path_matches() {
        let endpoints = vec![ep(HttpMethod::Get, "/orders")];
        assert!(matches!(
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users"),
            MockResolution::NotFound
        ));
    }

    #[test]
    fn resolve_returns_method_not_allowed_when_path_matches_but_method_differs() {
        let endpoints = vec![ep(HttpMethod::Post, "/users")];
        assert!(matches!(
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users"),
            MockResolution::MethodNotAllowed(_)
        ));
    }

    #[test]
    fn resolve_method_not_allowed_lists_supported_methods() {
        let endpoints = vec![
            ep(HttpMethod::Post, "/users"),
            ep(HttpMethod::Put, "/users"),
        ];
        if let MockResolution::MethodNotAllowed(allowed) =
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users")
        {
            assert_eq!(allowed.len(), 2);
        } else {
            panic!("expected MethodNotAllowed");
        }
    }

    #[test]
    fn resolve_exact_beats_wildcard() {
        let exact = ep(HttpMethod::Get, "/users/me");
        let wildcard = ep(HttpMethod::Get, "/users/{id}");
        let endpoints = vec![wildcard, exact.clone()];
        if let MockResolution::Matched(ep) =
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users/me")
        {
            assert_eq!(ep.id, exact.id);
        } else {
            panic!("expected Matched");
        }
    }

    #[test]
    fn resolve_wildcard_matches_when_no_exact() {
        let wild = ep(HttpMethod::Get, "/users/{id}");
        let endpoints = vec![wild.clone()];
        if let MockResolution::Matched(ep) =
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/users/42")
        {
            assert_eq!(ep.id, wild.id);
        } else {
            panic!("expected Matched");
        }
    }

    #[test]
    fn resolve_empty_endpoints_returns_not_found() {
        assert!(matches!(
            resolve_endpoint(&[], &HttpMethod::Get, "/anything"),
            MockResolution::NotFound
        ));
    }

    #[test]
    fn resolve_returns_correct_endpoint_among_multiple_methods() {
        let get_ep = ep(HttpMethod::Get, "/items");
        let post_ep = ep(HttpMethod::Post, "/items");
        let endpoints = vec![get_ep.clone(), post_ep];
        if let MockResolution::Matched(ep) =
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/items")
        {
            assert_eq!(ep.id, get_ep.id);
        } else {
            panic!("expected Matched");
        }
    }

    #[test]
    fn resolve_multi_segment_wildcard_path() {
        let ep1 = ep(HttpMethod::Get, "/a/{b}/c/{d}");
        let endpoints = vec![ep1.clone()];
        if let MockResolution::Matched(ep) =
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/a/1/c/2")
        {
            assert_eq!(ep.id, ep1.id);
        } else {
            panic!("expected Matched");
        }
    }

    #[test]
    fn resolve_multi_segment_wildcard_does_not_match_wrong_literal() {
        let ep1 = ep(HttpMethod::Get, "/a/{b}/c/{d}");
        let endpoints = vec![ep1];
        assert!(matches!(
            resolve_endpoint(&endpoints, &HttpMethod::Get, "/a/1/x/2"),
            MockResolution::NotFound
        ));
    }
}
