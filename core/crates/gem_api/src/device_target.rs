use gem_client::ClientError;
use primitives::ScanTransactionPayload;
use serde_json::value::{RawValue, to_raw_value};

use crate::method::GemApiMethod;

#[derive(Clone, Debug)]
pub enum GemDeviceApiTarget {
    ScanTransaction(ScanTransactionPayload),
}

impl GemDeviceApiTarget {
    pub fn method(&self) -> GemApiMethod {
        match self {
            Self::ScanTransaction(_) => GemApiMethod::Post,
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::ScanTransaction(_) => "/v2/devices/scan/transaction".to_string(),
        }
    }

    pub fn wallet_id(&self) -> &str {
        match self {
            Self::ScanTransaction(_) => "",
        }
    }

    pub fn body(&self) -> Result<Option<Box<RawValue>>, ClientError> {
        match self {
            Self::ScanTransaction(payload) => Ok(Some(to_raw_value(payload)?)),
        }
    }
}
