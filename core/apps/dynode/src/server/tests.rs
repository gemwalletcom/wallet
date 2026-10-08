use axum::body::Body;
use axum::response::Response;
use http::header::{CONTENT_TYPE, HOST};
use http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::*;
use crate::testkit::server_mock::TEST_REQUEST_LIMIT;

async fn send(router: &Router, method: Method, path: &str, body: Option<Vec<u8>>) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    if body.is_some() {
        request = request.header(HOST, "localhost");
    }
    router.clone().oneshot(request.body(body.map(Body::from).unwrap_or_default()).unwrap()).await.unwrap()
}

fn content_type(response: &Response) -> Option<&str> {
    response.headers().get(CONTENT_TYPE).and_then(|value| value.to_str().ok())
}

async fn text(response: Response) -> String {
    String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap()
}

async fn json(response: Response) -> Value {
    serde_json::from_str(&text(response).await).unwrap()
}

#[tokio::test]
async fn test_node_health_root_and_metrics() {
    let router = Server::mock_nodes().router();
    assert_eq!(send(&router, Method::GET, "/health", None).await.status(), StatusCode::OK);
    let root = send(&router, Method::GET, "/", None).await;
    assert_eq!(root.status(), StatusCode::OK);
    assert_eq!(text(root).await, "ok");
    let response = send(&router, Method::GET, "/metrics", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(content_type(&response), Some("text/plain; charset=utf-8"));
    let metrics = text(response).await;
    assert!(!metrics.contains("auth_requests"));
    assert_eq!(metrics.lines().filter(|line| line.starts_with("egress_")).count(), 0);
    assert!(metrics.contains("dynode_http_requests_total{method=\"GET\",route=\"/health\",status=\"200\"} 1"));
}

#[tokio::test]
async fn test_node_invalid_chain_and_missing_host_are_json_errors() {
    let router = Server::mock_nodes().router();
    for (path, message) in [("/invalid-chain", "Invalid chain"), ("/auth", "Invalid chain"), ("/ethereum", "Failed to build request")] {
        let response = send(&router, Method::GET, path, None).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(content_type(&response), Some("application/json"));
        assert_eq!(json(response).await, json!({ "error": { "message": message } }));
    }
    let response = send(&router, Method::POST, "/health", Some(b"{}".to_vec())).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(response).await, json!({ "error": { "message": "Invalid chain" } }));
    let response = send(&router, Method::TRACE, "/ethereum", None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(response).await, json!({ "error": { "message": "route not found" } }));
}

#[tokio::test]
async fn test_provider_health_and_route_access() {
    let router = Server::mock_egress().router();
    assert_eq!(send(&router, Method::GET, "/health", None).await.status(), StatusCode::OK);
    for (method, path, status, message) in [
        (Method::GET, "/", StatusCode::NOT_FOUND, "route not found"),
        (Method::GET, "/auth", StatusCode::NOT_FOUND, "route not found"),
        (Method::GET, "/worker/missing/allowed", StatusCode::NOT_FOUND, "route not found"),
        (Method::GET, "/worker/security_public/denied", StatusCode::FORBIDDEN, "request not allowed"),
        (Method::POST, "/worker/security_public/allowed", StatusCode::FORBIDDEN, "request not allowed"),
        (Method::GET, "/ethereum", StatusCode::NOT_FOUND, "route not found"),
    ] {
        let response = send(&router, method.clone(), path, None).await;
        assert_eq!(response.status(), status, "{method} {path}");
        assert_eq!(content_type(&response), Some("application/json"));
        assert_eq!(json(response).await, json!({ "error": { "message": message } }));
    }
}

#[tokio::test]
async fn test_egress_unavailable_endpoint_preserves_metrics() {
    let router = Server::mock_egress().router();
    let response = send(&router, Method::GET, "/worker/security_public/allowed?token=not-a-metric-label", None).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(content_type(&response), Some("application/json"));
    assert_eq!(json(response).await, json!({ "error": { "message": "no endpoint is available" } }));

    let response = send(&router, Method::GET, "/metrics", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(content_type(&response), Some("text/plain; charset=utf-8"));
    let metrics = text(response).await;
    assert_eq!(
        metrics.lines().filter(|line| line.starts_with("dynode_") && !line.starts_with("dynode_http_")).collect::<Vec<_>>(),
        vec![
            "dynode_responses_total{source=\"worker\",group=\"security\",service=\"security_public\",path=\"/allowed\",status=\"503\"} 1",
            "dynode_inflight{source=\"worker\",group=\"security\",service=\"security_public\"} 0",
        ]
    );
    assert_eq!(
        metrics.lines().filter(|line| line.starts_with("dynode_http_requests_total")).collect::<Vec<_>>(),
        vec!["dynode_http_requests_total{method=\"GET\",route=\"unmatched\",status=\"503\"} 1"]
    );
    assert_eq!(metrics.lines().filter(|line| line.starts_with("egress_")).count(), 0);
}

#[tokio::test]
async fn test_request_body_limit_accepts_exact_size_and_rejects_truncation_in_both_modes() {
    for (router, path, denied_message) in [
        (Server::mock_nodes().router(), "/ethereum/denied", "Request not allowed"),
        (Server::mock_egress().router(), "/worker/security_public/denied", "request not allowed"),
    ] {
        for (length, status, message) in [
            (TEST_REQUEST_LIMIT, StatusCode::FORBIDDEN, denied_message),
            (TEST_REQUEST_LIMIT + 1, StatusCode::PAYLOAD_TOO_LARGE, "request body is too large"),
        ] {
            let response = send(&router, Method::POST, path, Some(vec![b'x'; length])).await;
            assert_eq!(response.status(), status, "{path} body length {length}");
            assert_eq!(content_type(&response), Some("application/json"));
            assert_eq!(json(response).await, json!({ "error": { "message": message } }));
        }
    }
}

#[test]
fn test_mixed_routes_reserve_chain_prefixes() {
    let server = Server::new(Config::mock(), HashMap::from([(Chain::Ethereum, ChainConfig::mock(Chain::Ethereum))])).unwrap();

    assert!(matches!(server.routes.target("/ethereum"), Ok(Target::Node(_, Chain::Ethereum))));
    assert!(matches!(server.routes.target("/ethereum/fastnear_tx/v0/transactions"), Ok(Target::Node(_, Chain::Ethereum))));
    assert!(matches!(server.routes.target("/base/fastnear_tx/v0/transactions"), Ok(Target::Node(_, Chain::Base))));
    for source in ["api", "parser", "consumer", "daemon"] {
        assert!(matches!(server.routes.target(&format!("/{source}/fastnear_tx/v0/transactions")), Ok(Target::Provider(_))));
    }
}

#[test]
fn test_parse_chain() {
    assert_eq!(parse_chain("/solana"), Some(Chain::Solana));
    assert_eq!(parse_chain("/solana?dkey=abc123"), Some(Chain::Solana));
    assert_eq!(parse_chain("/solana/rpc?dkey=abc123"), Some(Chain::Solana));
    assert_eq!(parse_chain("/ethereum/v1/rpc"), Some(Chain::Ethereum));
    assert_eq!(parse_chain("/invalid"), None);
    assert_eq!(parse_chain(""), None);
}
