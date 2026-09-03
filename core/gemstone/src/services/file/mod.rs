pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::services::error::GemServiceError;
pub use store::GemFileStore;

pub(crate) async fn download(provider: &Arc<dyn AlienProvider>, url: String) -> Result<Vec<u8>, GemServiceError> {
    let target = AlienTarget {
        url,
        method: AlienHttpMethod::Get,
        headers: None,
        body: None,
    };
    let response = provider
        .request(target)
        .await
        .map_err(|error| GemServiceError::Api { msg: error.to_string() })?
        .to_rpc_response();
    if let Some(status) = response.status
        && !(200..300).contains(&status)
    {
        return Err(GemServiceError::Api {
            msg: format!("download failed with status {status}"),
        });
    }
    Ok(response.data)
}
