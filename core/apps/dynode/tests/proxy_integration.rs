use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Cursor;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, str};

use dynode::BoxError;
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use reqwest::{Client, Method, Url};
use rocket::config::Config as RocketConfig;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{Method as RocketMethod, Status};
use rocket::response::Response;
use rocket::route::{Handler, Outcome, Route};
use rocket::{Request, Shutdown};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;

const RAW_BODY: &[u8] = b"\0raw-proxy-body\xff";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct Echo {
    path: String,
    query: BTreeMap<String, String>,
    body: String,
    authorization: Option<String>,
    variant: Option<String>,
    dropped: Option<String>,
    sequence: usize,
}

struct RecordedRequest {
    echo: Echo,
    received: Instant,
}

#[derive(Clone, Default)]
struct Upstream(Arc<Mutex<Vec<RecordedRequest>>>);

impl Upstream {
    fn count(&self, path: &str) -> usize {
        self.0.lock().unwrap().iter().filter(|request| request.echo.path == path).count()
    }
}

#[rocket::async_trait]
impl Handler for Upstream {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let url = Url::parse(&format!("http://fixture{}", request.uri())).unwrap();
        let body = data.open(64.kibibytes()).into_bytes().await.unwrap().into_inner();
        let echo = {
            let mut records = self.0.lock().unwrap();
            let echo = Echo {
                path: url.path().to_string(),
                query: url.query_pairs().map(|(name, value)| (name.into_owned(), value.into_owned())).collect(),
                body: String::from_utf8(body.clone()).unwrap(),
                authorization: request.headers().get_one("Authorization").map(str::to_string),
                variant: request.headers().get_one("X-Variant").map(str::to_string),
                dropped: request.headers().get_one("X-Do-Not-Forward").map(str::to_string),
                sequence: records.len(),
            };
            records.push(RecordedRequest {
                echo: echo.clone(),
                received: Instant::now(),
            });
            echo
        };
        let (status, content_type, body) = if url.path().starts_with("/node/") {
            let call: Value = serde_json::from_slice(&body).unwrap();
            if call["method"] == "eth_blockNumber" {
                sleep(Duration::from_millis(750)).await;
            }
            let result = if call["method"] == "eth_getCode" {
                "a".repeat(4096)
            } else if url.path() == "/node/region" {
                "0x1".to_string()
            } else {
                "0xbad".to_string()
            };
            let body = format!(r#"{{"jsonrpc":"2.0","id":{},"result":"{result}"}}"#, call["id"]);
            (Status::Ok, "application/json", body.into_bytes())
        } else if url.path().starts_with("/first429/") || url.path().starts_with("/last429/") {
            (Status::TooManyRequests, "text/plain", url.path().as_bytes().to_vec())
        } else if url.path() == "/echo/status" {
            (Status::ImATeapot, "application/octet-stream", RAW_BODY.to_vec())
        } else {
            if url.path() == "/echo/slow" {
                sleep(Duration::from_millis(750)).await;
            }
            (Status::Ok, "application/json", serde_json::to_vec(&echo).unwrap())
        };
        let mut response = Response::build();
        response
            .status(status)
            .raw_header("Content-Type", content_type)
            .raw_header("X-Provider", "fixture")
            .raw_header("X-Do-Not-Forward", "upstream-private")
            .sized_body(body.len(), Cursor::new(body));
        if status == Status::TooManyRequests {
            response.raw_header("Retry-After", "60");
        }
        if url.path() == "/privacy/no-store" {
            response.raw_header("Cache-Control", "no-store");
        }
        if url.path() == "/privacy/cookie" {
            response.raw_header("Set-Cookie", "session=test");
        }
        Outcome::Success(response.finalize())
    }
}

enum Process {
    Native(Child),
    Docker(String),
}

struct Harness {
    directory: PathBuf,
    process: Option<Process>,
    upstream: Upstream,
    upstream_shutdown: Shutdown,
    upstream_task: JoinHandle<()>,
    client: Client,
    base: String,
}

struct Reply {
    headers: HeaderMap,
    body: Vec<u8>,
}

fn free_port() -> Result<u16, BoxError> {
    Ok(TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port())
}

impl Harness {
    async fn start() -> Result<Self, BoxError> {
        let image = env::var("DYNODE_TEST_IMAGE").ok();
        let upstream_port = free_port()?;
        let port = free_port()?;
        let upstream = Upstream::default();
        let fixture = rocket::custom(
            RocketConfig::figment()
                .merge(("address", if image.is_some() { "0.0.0.0" } else { "127.0.0.1" }))
                .merge(("port", upstream_port))
                .merge(("log_level", "off")),
        )
        .mount("/", [RocketMethod::Get, RocketMethod::Post].map(|method| Route::new(method, "/<path..>", upstream.clone())))
        .ignite()
        .await?;
        let directory = env::temp_dir().join(format!("dynode-integration-{}", Uuid::new_v4()));
        fs::create_dir(&directory)?;
        let shutdown = fixture.shutdown();
        let mut harness = Self {
            directory,
            process: None,
            upstream,
            upstream_shutdown: shutdown,
            upstream_task: tokio::spawn(async move {
                fixture.launch().await.unwrap();
            }),
            client: Client::builder().no_proxy().timeout(Duration::from_secs(10)).build()?,
            base: format!("http://127.0.0.1:{port}"),
        };
        harness.write_config(upstream_port, if image.is_some() { 3000 } else { port }, image.is_some())?;
        if let Some(image) = image {
            let container = format!("dynode-integration-{}", Uuid::new_v4());
            let mut command = Command::new("docker");
            command.args(["run", "--detach", "--name", &container, "-p", &format!("127.0.0.1:{port}:3000")]);
            command.args([
                "-v",
                &format!("{}:/integration:ro", harness.directory.display()),
                "-e",
                "DYNODE_CONFIG=/integration/config.yml",
            ]);
            if !cfg!(target_os = "macos") {
                command.args(["--add-host", "host.docker.internal:host-gateway"]);
            }
            harness.process = Some(Process::Docker(container));
            let output = command.arg(image).output()?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
            }
        } else {
            let log = File::create(harness.directory.join("server.log"))?;
            let child = Command::new(env!("CARGO_BIN_EXE_dynode"))
                .current_dir(&harness.directory)
                .env("DYNODE_CONFIG", harness.directory.join("config.yml"))
                .stdout(Stdio::from(log.try_clone()?))
                .stderr(Stdio::from(log))
                .spawn()?;
            harness.process = Some(Process::Native(child));
        }
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            if let Ok(response) = harness.client.get(format!("{}/health", harness.base)).send().await
                && response.status().is_success()
            {
                return Ok(harness);
            }
            let exited = match &mut harness.process {
                Some(Process::Native(child)) => child.try_wait()?.is_some(),
                Some(Process::Docker(_)) | None => false,
            };
            if exited || Instant::now() >= deadline {
                return Err(format!("Dynode did not become ready: {}", harness.logs()?).into());
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    fn write_config(&self, upstream_port: u16, port: u16, docker: bool) -> Result<(), BoxError> {
        let host = if docker { "host.docker.internal" } else { "127.0.0.1" };
        let upstream = format!("http://{host}:{upstream_port}");
        let basic = format!("http://smoke-user:smoke-password@{host}:{upstream_port}");
        for (name, content) in [
            ("config.yml", include_str!("../testdata/proxy_integration/config.yml")),
            ("routes.yml", include_str!("../testdata/proxy_integration/routes.yml")),
            ("chains.base.yml", include_str!("../testdata/proxy_integration/chains.base.yml")),
            ("chains.region.yml", include_str!("../testdata/proxy_integration/chains.region.yml")),
        ] {
            fs::write(
                self.directory.join(name),
                content
                    .replace("__PORT__", &port.to_string())
                    .replace("__UPSTREAM__", &upstream)
                    .replace("__UPSTREAM_BASIC__", &basic),
            )?;
        }
        Ok(())
    }

    async fn call(&self, path: &str, status: u16, body: Option<&[u8]>, headers: &[(&str, &str)]) -> Result<Reply, BoxError> {
        let mut request = self
            .client
            .request(if body.is_some() { Method::POST } else { Method::GET }, format!("{}{path}", self.base))
            .header(CONTENT_TYPE, "application/json");
        if let Some(body) = body {
            request = request.body(body.to_vec());
        }
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await?;
        let actual = response.status().as_u16();
        let headers = response.headers().clone();
        let body = response.bytes().await?.to_vec();
        assert_eq!(actual, status, "{path}: {}", String::from_utf8_lossy(&body));
        Ok(Reply { headers, body })
    }

    fn logs(&self) -> Result<String, BoxError> {
        match &self.process {
            Some(Process::Native(_)) => Ok(fs::read_to_string(self.directory.join("server.log"))?),
            Some(Process::Docker(container)) => {
                let output = Command::new("docker").args(["logs", container]).output()?;
                Ok(format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))
            }
            None => Ok(String::new()),
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        match &mut self.process {
            Some(Process::Native(child)) => {
                let _ = child.kill();
                let _ = child.wait();
            }
            Some(Process::Docker(container)) => {
                let _ = Command::new("docker").args(["rm", "--force", container]).output();
            }
            None => {}
        }
        self.upstream_shutdown.clone().notify();
        self.upstream_task.abort();
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn provider(service: &str, path: &str) -> String {
    format!("/worker/providers_{service}{path}")
}

#[tokio::test(flavor = "multi_thread")]
async fn test_proxy_routing() -> Result<(), BoxError> {
    let harness = Harness::start().await?;
    assert_eq!(harness.call("/", 200, None, &[]).await?.body, b"ok");
    harness.call("/auth", 404, None, &[]).await?;
    for id in [7, 19] {
        let body = format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"eth_chainId","params":[]}}"#);
        let response = harness.call("/ethereum", 200, Some(body.as_bytes()), &[]).await?;
        assert_eq!(
            serde_json::from_slice::<Value>(&response.body)?,
            serde_json::from_str::<Value>(&format!(r#"{{"jsonrpc":"2.0","id":{id},"result":"0x1"}}"#))?
        );
    }
    assert_eq!((harness.upstream.count("/node/base"), harness.upstream.count("/node/region")), (0, 1));
    harness
        .call("/ethereum", 403, Some(br#"{"jsonrpc":"2.0","id":1,"method":"notAllowed","params":[]}"#), &[])
        .await?;
    for path in ["/worker/missing/inspect", "/worker/providers/echo/inspect"] {
        harness.call(path, 404, None, &[]).await?;
    }
    let denied = harness.call(&provider("allow", "/denied"), 403, None, &[]).await?;
    assert_eq!(denied.headers.get(CONTENT_TYPE).unwrap(), "application/json");
    assert_eq!(
        serde_json::from_slice::<Value>(&denied.body)?,
        serde_json::from_str::<Value>(r#"{"error":{"message":"request not allowed"}}"#)?
    );
    harness.call(&provider("allow", "/allowed"), 403, Some(b"{}"), &[]).await?;
    for (path, limit) in [("/ethereum/denied".to_string(), 1024), (provider("allow", "/denied"), 2048)] {
        harness.call(&path, 403, Some(&vec![b'x'; limit]), &[]).await?;
        harness.call(&path, 413, Some(&vec![b'x'; limit + 1]), &[]).await?;
    }
    let response = harness
        .call(
            &provider("echo", "/inspect?apikey=caller-key&query=kept"),
            200,
            Some(br#"{"hello":"world"}"#),
            &[("Authorization", "Bearer caller-key"), ("X-Do-Not-Forward", "inbound-private")],
        )
        .await?;
    let echo: Echo = serde_json::from_slice(&response.body)?;
    assert_eq!(
        (echo.authorization.as_deref(), echo.body.as_str(), echo.dropped),
        (Some("Bearer endpoint-key"), r#"{"hello":"world"}"#, None)
    );
    assert_eq!(echo.query, BTreeMap::from([("apikey".into(), "configured-key".into()), ("query".into(), "kept".into())]));
    assert_eq!(response.headers.get("x-provider").unwrap(), "fixture");
    assert_eq!(response.headers.get("x-do-not-forward"), None);
    assert_eq!(harness.call(&provider("echo", "/status"), 418, None, &[]).await?.body, RAW_BODY);
    let basic: Echo = serde_json::from_slice(&harness.call(&provider("basic", "/inspect"), 200, None, &[]).await?.body)?;
    assert_eq!(basic.authorization.as_deref(), Some("Basic c21va2UtdXNlcjpzbW9rZS1wYXNzd29yZA=="));
    for _ in 0..2 {
        harness.call(&provider("failover", "/quote"), 200, None, &[]).await?;
    }
    assert_eq!((harness.upstream.count("/first429/quote"), harness.upstream.count("/fallback/quote")), (1, 2));
    for _ in 0..2 {
        harness.call(&provider("exhausted", "/quote"), 429, None, &[]).await?;
    }
    assert_eq!((harness.upstream.count("/first429/quote"), harness.upstream.count("/last429/quote")), (2, 1));
    for _ in 0..3 {
        harness.call(&provider("paced", "/data"), 200, None, &[]).await?;
    }
    let times = harness
        .upstream
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|record| record.echo.path == "/paced/data")
        .map(|record| record.received)
        .collect::<Vec<_>>();
    assert_eq!(times.len(), 3);
    assert!(times.windows(2).all(|pair| pair[1].duration_since(pair[0]) >= Duration::from_millis(200)));
    verify_cache(&harness).await?;
    verify_metrics(&harness).await?;
    verify_family_policies(&harness).await?;
    Ok(())
}

async fn verify_cache(harness: &Harness) -> Result<(), BoxError> {
    let path = provider("cache", "/value?q=a");
    let first = harness.call(&path, 200, Some(b"{}"), &[("X-Variant", "one")]).await?;
    let second = harness.call(&path, 200, Some(b"{}"), &[("X-Variant", "one")]).await?;
    assert_eq!(first.body, second.body);
    assert_eq!(second.headers.get("x-provider").unwrap(), "fixture");
    assert_eq!(harness.upstream.count("/cache/value"), 1);
    for (target, body, variant) in [
        (provider("cache", "/value?q=b"), b"{}".as_slice(), "one"),
        (path.clone(), br#"{"changed":true}"#.as_slice(), "one"),
        (path, b"{}".as_slice(), "two"),
        ("/other/providers_cache/value?q=a".to_string(), b"{}".as_slice(), "one"),
    ] {
        assert_ne!(harness.call(&target, 200, Some(body), &[("X-Variant", variant)]).await?.body, first.body);
    }
    assert_eq!(harness.upstream.count("/cache/value"), 5);
    harness.call("/worker/missing/cache/value?q=a", 404, Some(b"{}"), &[("X-Variant", "one")]).await?;
    assert_eq!(harness.upstream.count("/cache/value"), 5);
    for (suffix, header) in [("/no-store", "cache-control"), ("/cookie", "set-cookie")] {
        let first = harness.call(&provider("privacy", suffix), 200, None, &[]).await?;
        let second = harness.call(&provider("privacy", suffix), 200, None, &[]).await?;
        assert_ne!(first.body, second.body);
        assert_eq!(harness.upstream.count(&format!("/privacy{suffix}")), 2);
        assert_eq!((first.headers.get(header), second.headers.get(header)), (None, None));
    }
    Ok(())
}

async fn verify_metrics(harness: &Harness) -> Result<(), BoxError> {
    let metrics = String::from_utf8(harness.call("/metrics", 200, None, &[]).await?.body)?;
    for expected in [
        "dynode_requests_total{source=\"worker\",group=\"providers\",service=\"providers_echo\",endpoint=\"endpoint_0\",path=\"/inspect\",status=\"200\"} 1",
        "dynode_requests_total{source=\"worker\",group=\"providers\",service=\"providers_failover\",endpoint=\"endpoint_0\",path=\"/quote\",status=\"429\"} 1",
        "dynode_requests_total{source=\"worker\",group=\"providers\",service=\"providers_failover\",endpoint=\"endpoint_1\",path=\"/quote\",status=\"200\"} 2",
        "dynode_requests_total{source=\"worker\",group=\"providers\",service=\"providers_cache\",endpoint=\"endpoint_0\",path=\"/value\",status=\"200\"} 4",
        "dynode_requests_total{source=\"other\",group=\"providers\",service=\"providers_cache\",endpoint=\"endpoint_0\",path=\"/value\",status=\"200\"} 1",
    ] {
        assert_eq!(metrics.lines().find(|line| *line == expected), Some(expected));
    }
    for credential in ["configured-key", "endpoint-key"] {
        assert_eq!(metrics.find(credential), None);
    }
    Ok(())
}

async fn verify_family_policies(harness: &Harness) -> Result<(), BoxError> {
    let slow_call = br#"{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}"#;
    harness.call("/ethereum", 500, Some(slow_call), &[]).await?;
    harness.call(&provider("echo", "/slow"), 200, None, &[]).await?;

    let before = harness.upstream.count("/node/region");
    let large_call = br#"{"jsonrpc":"2.0","id":1,"method":"eth_getCode","params":[]}"#;
    for _ in 0..2 {
        let response = harness.call("/ethereum", 200, Some(large_call), &[]).await?;
        assert_eq!(serde_json::from_slice::<Value>(&response.body)?["result"].as_str().unwrap().len(), 4096);
    }
    assert_eq!(harness.upstream.count("/node/region"), before + 2);

    let large_body = vec![b'x'; 1500];
    let path = provider("cache", "/value?q=large");
    let before = harness.upstream.count("/cache/value");
    let first = harness.call(&path, 200, Some(&large_body), &[]).await?;
    let second = harness.call(&path, 200, Some(&large_body), &[]).await?;
    assert_eq!(first.body, second.body);
    assert_eq!(harness.upstream.count("/cache/value"), before + 1);
    Ok(())
}
