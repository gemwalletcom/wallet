use rocket::Request;
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome};
use services::devices::DeviceRecord;

use super::auth::{authenticate, lookup_device};

// Verifies the device request signature, then checks that the device exists.
#[derive(Clone)]
pub struct AuthenticatedDevice {
    pub record: DeviceRecord,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        req.local_cache_async(async {
            let auth = match authenticate(req).await {
                Ok(auth) => auth,
                Err(error) => return error,
            };

            let record = match lookup_device(req, &auth.device_id).await {
                Ok(result) => result,
                Err(error) => return error,
            };

            Success(AuthenticatedDevice { record })
        })
        .await
        .clone()
    }
}
