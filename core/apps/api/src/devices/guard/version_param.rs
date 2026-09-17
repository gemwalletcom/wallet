use primitives::Version;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};

use super::AuthenticatedDevice;

pub struct VersionParam(pub Version);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VersionParam {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<AuthenticatedDevice>().await {
            Outcome::Success(auth) => match auth.device_row.version.parse::<Version>() {
                Ok(version) => Outcome::Success(Self(version)),
                Err(error) => Outcome::Error((Status::BadRequest, error.to_string())),
            },
            Outcome::Error(error) => Outcome::Error(error),
            Outcome::Forward(status) => Outcome::Forward(status),
        }
    }
}
