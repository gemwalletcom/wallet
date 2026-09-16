use std::io::Cursor;

use primitives::ResponseResult;
use rocket::Request;
use rocket::http::Status;
use rocket::response::{Responder, Response, Result as ResponderResult};
use rocket::serde::json::Json;

use crate::proxy::ProxyResponse;

impl<'r> Responder<'r, 'static> for ProxyResponse {
    fn respond_to(self, _: &'r Request<'_>) -> ResponderResult<'static> {
        let ProxyResponse { status, headers, body, .. } = self;

        let mut builder = Response::build();
        let status = Status::new(status);
        builder.status(status);

        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                builder.raw_header(name.as_str().to_string(), value_str.to_string());
            }
        }

        let body_len = body.len();
        builder.sized_body(body_len, Cursor::new(body));
        Ok(builder.finalize())
    }
}

pub(crate) struct ProxyError {
    status: Status,
    message: String,
}

impl ProxyError {
    pub(crate) fn new(status: Status, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}

impl<'r> Responder<'r, 'static> for ProxyError {
    fn respond_to(self, request: &'r Request<'_>) -> ResponderResult<'static> {
        let response = Json(ResponseResult::<()>::error(self.message));
        Response::build_from(response.respond_to(request)?).status(self.status).ok()
    }
}
