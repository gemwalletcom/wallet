use std::error::Error;

use primitives::ValueAccess;
use serde_json::Value;

use crate::models::BroadcastTransaction;
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response: Value = serde_json::from_str(response)?;
    // TODO(2027-01-01): Remove v2 decoding when Dynode drops legacy wallet routes.
    if let Some(result) = response.get("result") {
        if response.get("ok") != Some(&Value::Bool(true)) {
            return Err("TON broadcast rejected".into());
        }
        return map_transaction_broadcast(BroadcastTransaction {
            message_hash: result.get_string("hash")?.to_string(),
        });
    }
    map_transaction_broadcast(serde_json::from_value(response)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcast_response_versions() {
        for response in [
            r#"{"message_hash":"gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY="}"#,
            r#"{"ok":true,"result":{"hash":"gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY="}}"#,
        ] {
            assert_eq!(map_transaction_broadcast_response_from_str(response).unwrap(), "8328eaffb209e4aa52bd9967c22c5a4b7463236c64d7ee69ba9d24fbe4bfc976");
        }
        for response in [
            r#"{"ok":false,"result":{"hash":"gyjq/7IJ5KpSvZlnwixaS3RjI2xk1+5pup0k++S/yXY="}}"#,
            r#"{"error":"invalid BOC"}"#,
            r#"{"message_hash":"invalid base64"}"#,
        ] {
            assert!(map_transaction_broadcast_response_from_str(response).is_err());
        }
    }
}
