use std::error::Error;
use std::io::Cursor;
use std::net::IpAddr;

use gem_tracing::info_with_fields;
use reqwest::Method;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::config::Config;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{ContentType, Method as RocketMethod, Status};
use rocket::outcome::Outcome as RequestOutcome;
use rocket::response::content::RawText;
use rocket::response::{Responder, Response};
use rocket::route::{Handler, Outcome, Route};
use rocket::{Request, State};

use crate::gateway::{Gateway, GatewayError, GatewayResponse};
use crate::metrics::Metrics;

#[derive(Clone)]
struct EgressHandler {
    limit: usize,
}

#[rocket::async_trait]
impl Handler for EgressHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let gateway = match request.guard::<&State<Gateway>>().await {
            RequestOutcome::Success(state) => state,
            RequestOutcome::Error((status, ())) | RequestOutcome::Forward(status) => {
                return Outcome::from(request, GatewayError::new(status, "gateway state is unavailable"));
            }
        };
        match forward_request(request, data, gateway.inner(), self.limit).await {
            Ok(response) => Outcome::from(request, response),
            Err(error) => Outcome::from(request, error),
        }
    }
}

async fn forward_request(request: &Request<'_>, data: Data<'_>, gateway: &Gateway, limit: usize) -> Result<GatewayResponse, GatewayError> {
    let method = Method::from_bytes(request.method().as_str().as_bytes()).map_err(|_| GatewayError::new(Status::BadRequest, "invalid HTTP method"))?;
    let body = read_request_body(data, limit).await?;
    let headers = request_headers(request)?;
    gateway.forward(method, &request.uri().to_string(), &headers, body).await
}

async fn read_request_body(data: Data<'_>, limit: usize) -> Result<Vec<u8>, GatewayError> {
    let body = data
        .open(limit.bytes())
        .into_bytes()
        .await
        .map_err(|error| GatewayError::new(Status::BadRequest, error.to_string()))?;
    if !body.is_complete() {
        return Err(GatewayError::new(Status::PayloadTooLarge, "request body is too large"));
    }
    Ok(body.into_inner())
}

fn request_headers(request: &Request<'_>) -> Result<HeaderMap, GatewayError> {
    let mut headers = HeaderMap::new();
    for header in request.headers().iter() {
        let name = HeaderName::from_bytes(header.name().as_str().as_bytes()).map_err(|error| GatewayError::new(Status::BadRequest, error.to_string()))?;
        let value = HeaderValue::from_str(header.value()).map_err(|error| GatewayError::new(Status::BadRequest, error.to_string()))?;
        headers.append(name, value);
    }
    Ok(headers)
}

fn egress_routes(limit: usize) -> Vec<Route> {
    [
        RocketMethod::Get,
        RocketMethod::Post,
        RocketMethod::Put,
        RocketMethod::Patch,
        RocketMethod::Delete,
        RocketMethod::Options,
        RocketMethod::Head,
        RocketMethod::Trace,
        RocketMethod::Connect,
    ]
    .into_iter()
    .map(|method| Route::new(method, "/<path..>", EgressHandler { limit }))
    .collect()
}

#[rocket::get("/health")]
fn health_endpoint() -> Status {
    Status::Ok
}

#[rocket::get("/metrics")]
fn metrics_endpoint(metrics: &State<Metrics>) -> RawText<String> {
    RawText(metrics.encode())
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for GatewayResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut builder = Response::build();
        builder.status(Status::new(self.status));
        for (name, value) in &self.headers {
            if let Ok(value) = value.to_str() {
                builder.raw_header(name.as_str().to_string(), value.to_string());
            }
        }
        builder.sized_body(self.body.len(), Cursor::new(self.body));
        Ok(builder.finalize())
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for GatewayError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let body = self.message;
        Response::build()
            .status(self.status)
            .header(ContentType::Plain)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

pub(crate) async fn launch(address: IpAddr, port: u16, limit: usize, gateway: Gateway, metrics: Metrics) -> Result<(), Box<dyn Error + Send + Sync>> {
    let server = rocket::custom(Config::figment().merge(("address", address)).merge(("port", port)))
        .manage(gateway)
        .manage(metrics)
        .mount("/", egress_routes(limit))
        .mount("/", rocket::routes![health_endpoint, metrics_endpoint])
        .ignite()
        .await?;
    info_with_fields!("Egress started", address = &format!("{address}:{port}"));
    server.launch().await?;
    Ok(())
}
