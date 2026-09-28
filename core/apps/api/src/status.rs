use primitives::unix_milliseconds;
use rocket::{get, http::Status as HttpStatus, serde::Serialize, serde::json::Json};

#[get("/")]
pub fn get_status(ip: std::net::IpAddr) -> Json<Status> {
    Json(Status {
        time: unix_milliseconds().unwrap_or_default(),
        ipv4: ip.to_string(),
    })
}

#[get("/health")]
pub fn get_health() -> HttpStatus {
    HttpStatus::Ok
}

#[derive(Serialize)]
pub struct Status {
    time: u64,
    ipv4: String,
}
