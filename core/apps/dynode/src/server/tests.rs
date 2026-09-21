use rocket::http::{ContentType, Header};
use rocket::local::asynchronous::Client;
use serde_json::{Value, json};

use super::*;
use crate::testkit::server_mock::TEST_REQUEST_LIMIT;

#[tokio::test]
async fn test_node_health_root_and_metrics() {
    let client = Client::tracked(Server::mock_nodes().rocket()).await.unwrap();
    assert_eq!(client.get("/health").dispatch().await.status(), Status::Ok);
    let root = client.get("/").dispatch().await;
    assert_eq!(root.status(), Status::Ok);
    assert_eq!(root.into_string().await.unwrap(), "ok");
    let response = client.get("/metrics").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    let metrics = response.into_string().await.unwrap();
    assert!(!metrics.contains("auth_requests"));
    assert_eq!(metrics.lines().filter(|line| line.starts_with("egress_")).count(), 0);
}

#[tokio::test]
async fn test_node_invalid_chain_and_missing_host_are_json_errors() {
    let client = Client::tracked(Server::mock_nodes().rocket()).await.unwrap();
    for (path, message) in [("/invalid-chain", "Invalid chain"), ("/auth", "Invalid chain"), ("/ethereum", "Failed to build request")] {
        let response = client.get(path).dispatch().await;
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(), json!({ "error": { "message": message } }));
    }
}

#[tokio::test]
async fn test_provider_health_and_route_access() {
    let client = Client::tracked(Server::mock_egress().rocket()).await.unwrap();
    assert_eq!(client.get("/health").dispatch().await.status(), Status::Ok);
    for (method, path, status, message) in [
        (RocketMethod::Get, "/", Status::NotFound, "route not found"),
        (RocketMethod::Get, "/auth", Status::NotFound, "route not found"),
        (RocketMethod::Get, "/worker/missing/allowed", Status::NotFound, "route not found"),
        (RocketMethod::Get, "/worker/security_public/denied", Status::Forbidden, "request not allowed"),
        (RocketMethod::Post, "/worker/security_public/allowed", Status::Forbidden, "request not allowed"),
        (RocketMethod::Get, "/ethereum", Status::NotFound, "route not found"),
    ] {
        let response = client.req(method, path).dispatch().await;
        assert_eq!(response.status(), status, "{method} {path}");
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(), json!({ "error": { "message": message } }));
    }
}

#[tokio::test]
async fn test_egress_unavailable_endpoint_preserves_metrics() {
    let client = Client::tracked(Server::mock_egress().rocket()).await.unwrap();
    let response = client.get("/worker/security_public/allowed?token=not-a-metric-label").dispatch().await;
    assert_eq!(response.status(), Status::ServiceUnavailable);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert_eq!(serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(), json!({ "error": { "message": "no endpoint is available" } }));

    let response = client.get("/metrics").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));
    let metrics = response.into_string().await.unwrap();
    assert_eq!(
        metrics.lines().filter(|line| line.starts_with("dynode_")).collect::<Vec<_>>(),
        vec![
            "dynode_responses_total{source=\"worker\",group=\"security\",service=\"security_public\",path=\"/allowed\",status=\"503\"} 1",
            "dynode_inflight{source=\"worker\",group=\"security\",service=\"security_public\"} 0",
        ]
    );
    assert_eq!(metrics.lines().filter(|line| line.starts_with("egress_")).count(), 0);
}

#[tokio::test]
async fn test_request_body_limit_accepts_exact_size_and_rejects_truncation_in_both_modes() {
    for (client, path, denied_message) in [
        (Client::tracked(Server::mock_nodes().rocket()).await.unwrap(), "/ethereum/denied", "Request not allowed"),
        (Client::tracked(Server::mock_egress().rocket()).await.unwrap(), "/worker/security_public/denied", "request not allowed"),
    ] {
        for (length, status, message) in [(TEST_REQUEST_LIMIT, Status::Forbidden, denied_message), (TEST_REQUEST_LIMIT + 1, Status::PayloadTooLarge, "request body is too large")] {
            let response = client.post(path).header(Header::new("Host", "localhost")).body(vec![b'x'; length]).dispatch().await;
            assert_eq!(response.status(), status, "{path} body length {length}");
            assert_eq!(response.content_type(), Some(ContentType::JSON));
            let body = response.into_string().await.unwrap();
            assert_eq!(serde_json::from_str::<Value>(&body).unwrap(), json!({ "error": { "message": message } }));
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
