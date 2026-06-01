use rocket::Request;
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome};

use super::auth::authenticate;

// Verifies the device request signature without checking the database.
pub struct VerifiedDeviceId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VerifiedDeviceId {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        match authenticate(req).await {
            Ok(auth) => Success(VerifiedDeviceId(auth.device_id)),
            Err(error) => error,
        }
    }
}
