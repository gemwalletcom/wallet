use config::{Config as FileConfig, File, FileFormat};
use rocket::http::{ContentType, Header};
use rocket::local::asynchronous::Client;
use serde_json::{Value, json};

use super::*;
use crate::config::RoutesConfig;
use crate::testkit::config::chain_config;

const TEST_LIMIT: usize = 8;

fn sample_config() -> Config {
    let mut config: Config = FileConfig::builder()
        .add_source(File::from_str(include_str!("../../config.yml"), FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    config.chains = Some(
        FileConfig::builder()
            .add_source(File::from_str(include_str!("../../chains.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap(),
    );
    config.routes = Some(
        FileConfig::builder()
            .add_source(File::from_str(include_str!("../../routes.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap(),
    );
    config
}

async fn nodes() -> Client {
    let mut config = sample_config();
    let settings = config.chains.as_mut().unwrap();
    settings.request.limit = TEST_LIMIT;
    settings.monitoring.enabled = false;
    config.routes = None;
    let chains = HashMap::from([(Chain::Ethereum, chain_config(Chain::Ethereum, "https://upstream.example.invalid"))]);
    Client::tracked(Server::new(config, chains).unwrap().rocket()).await.unwrap()
}

async fn egress() -> Client {
    let mut config = sample_config();
    config.chains = None;
    let mut settings: RoutesConfig = FileConfig::builder()
        .add_source(File::from_str(include_str!("../../testdata/route_headers.yml"), FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();
    settings.request.limit = TEST_LIMIT;
    settings.routes.get_mut("security_public").unwrap().allowlist = serde_json::from_value(json!([{ "path": "/allowed", "method": "GET" }])).unwrap();
    config.routes = Some(settings);
    Client::tracked(Server::new(config, HashMap::new()).unwrap().rocket()).await.unwrap()
}

#[tokio::test]
async fn test_node_health_root_and_metrics() {
    let client = nodes().await;
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
    let client = nodes().await;
    for (path, message) in [("/invalid-chain", "Invalid chain"), ("/auth", "Invalid chain"), ("/ethereum", "Failed to build request")] {
        let response = client.get(path).dispatch().await;
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(),
            json!({ "error": { "message": message } })
        );
    }
}

#[tokio::test]
async fn test_provider_health_and_route_access() {
    let client = egress().await;
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
        assert_eq!(
            serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(),
            json!({ "error": { "message": message } })
        );
    }
}

#[tokio::test]
async fn test_egress_unavailable_endpoint_preserves_metrics() {
    let client = egress().await;
    let response = client.get("/worker/security_public/allowed?token=not-a-metric-label").dispatch().await;
    assert_eq!(response.status(), Status::ServiceUnavailable);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert_eq!(
        serde_json::from_str::<Value>(&response.into_string().await.unwrap()).unwrap(),
        json!({ "error": { "message": "no endpoint is available" } })
    );

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
        (nodes().await, "/ethereum/denied", "Request not allowed"),
        (egress().await, "/worker/security_public/denied", "request not allowed"),
    ] {
        for (length, status, message) in [
            (TEST_LIMIT, Status::Forbidden, denied_message),
            (TEST_LIMIT + 1, Status::PayloadTooLarge, "request body is too large"),
        ] {
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
    let config = sample_config();
    let chains = HashMap::from([(Chain::Ethereum, chain_config(Chain::Ethereum, "https://upstream.example.invalid"))]);
    let server = Server::new(config, chains).unwrap();

    assert!(matches!(server.routes.target("/ethereum"), Ok(Target::Node(_, Chain::Ethereum))));
    assert!(matches!(
        server.routes.target("/ethereum/fastnear_tx/v0/transactions"),
        Ok(Target::Node(_, Chain::Ethereum))
    ));
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
