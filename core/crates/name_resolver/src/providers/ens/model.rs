use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CcipRequest {
    pub sender: String,
    pub data: String,
}

#[derive(Deserialize)]
pub struct CcipResponse {
    pub data: String,
}
