mod json;
mod raw;

use rocket::data::{ByteUnit, Outcome};
use rocket::http::Status;
use rocket::outcome::Outcome::Error;
use rocket::{Data, Request};

use crate::devices::signature::verify_request_body_hash;
use crate::responders::cache_error;

pub use json::DeviceJson;
pub use raw::DeviceBody;

fn error_outcome<'r, T>(req: &'r Request<'_>, status: Status, message: impl Into<String>) -> Outcome<'r, T, String> {
    let message = message.into();
    cache_error(req, &message);
    Error((status, message))
}

pub(crate) async fn read_verified_body<'r>(req: &'r Request<'_>, data: Data<'r>, limit: ByteUnit) -> Result<Vec<u8>, (Status, String)> {
    let bytes = data.open(limit).into_bytes().await.map_err(|_| (Status::BadRequest, "Failed to read body".to_string()))?;
    if !bytes.is_complete() {
        return Err((Status::BadRequest, "Request body too large".to_string()));
    }

    let raw_body = bytes.into_inner();
    verify_request_body_hash(req, &raw_body).await?;
    Ok(raw_body)
}
