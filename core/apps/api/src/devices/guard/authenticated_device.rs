use rocket::Request;
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome};
use storage::models::DeviceRow;

use super::auth::{authenticate, lookup_device};

// Verifies the device request signature, then checks that the device exists.
pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let auth = match authenticate(req).await {
            Ok(auth) => auth,
            Err(error) => return error,
        };

        let (device_row, _) = match lookup_device(req, &auth.device_id).await {
            Ok(result) => result,
            Err(error) => return error,
        };

        Success(AuthenticatedDevice { device_row })
    }
}
