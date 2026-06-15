use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::outcome::Outcome::Success;
use rocket::{Data, Request};

use super::{error_outcome, read_verified_body};

pub struct DeviceBody<const MAX_BYTES: u64>(Vec<u8>);

impl<const MAX_BYTES: u64> DeviceBody<MAX_BYTES> {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

#[rocket::async_trait]
impl<'r, const MAX_BYTES: u64> FromData<'r> for DeviceBody<MAX_BYTES> {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        match read_verified_body(req, data, MAX_BYTES.bytes()).await {
            Ok(body) => Success(DeviceBody(body)),
            Err((status, message)) => error_outcome(req, status, message),
        }
    }
}
