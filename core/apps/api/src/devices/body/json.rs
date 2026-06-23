use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::Success;
use rocket::{Data, Request};
use serde::de::DeserializeOwned;

use super::{error_outcome, read_verified_body};

const MAX_DEVICE_JSON_BODY_BYTES: u64 = 1024 * 1024;

pub struct DeviceJson<T>(T);

impl<T> DeviceJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned + Send> FromData<'r> for DeviceJson<T> {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let raw_body = match read_verified_body(req, data, MAX_DEVICE_JSON_BODY_BYTES.bytes()).await {
            Ok(body) => body,
            Err((status, message)) => return error_outcome(req, status, message),
        };

        match serde_json::from_slice::<T>(&raw_body) {
            Ok(value) => Success(DeviceJson(value)),
            Err(_) => error_outcome(req, Status::BadRequest, "Invalid JSON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use gem_auth::build_device_auth_header;
    use rocket::http::{ContentType, Header, Status};
    use rocket::local::blocking::Client;
    use rocket::{Build, Rocket, post, routes};

    use super::DeviceJson;
    use crate::devices::auth_config::AuthConfig;
    use gem_auth::AUTHORIZATION_HEADER;

    #[post("/", format = "json", data = "<body>")]
    async fn echo(body: DeviceJson<serde_json::Value>) -> &'static str {
        let _ = body;
        "ok"
    }

    fn rocket() -> Rocket<Build> {
        rocket::build().manage(AuthConfig::mock()).mount("/", routes![echo])
    }

    fn authorization(body: &[u8]) -> Header<'static> {
        let timestamp_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        Header::new(AUTHORIZATION_HEADER, build_device_auth_header(&[1u8; 32], "POST", "/", "", body, timestamp_ms).unwrap())
    }

    #[test]
    fn test_from_data_verifies_body_hash() {
        let client = Client::tracked(rocket()).unwrap();
        let body = br#"{"value":"ok"}"#;

        assert_eq!(
            client
                .post("/")
                .header(ContentType::JSON)
                .header(authorization(body))
                .body(body.as_slice())
                .dispatch()
                .status(),
            Status::Ok
        );
        assert_eq!(
            client
                .post("/")
                .header(ContentType::JSON)
                .header(authorization(body))
                .body(br#"{"value":"tampered"}"#.as_slice())
                .dispatch()
                .status(),
            Status::BadRequest
        );
    }
}
