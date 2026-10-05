use std::error::Error;

use primitives::graphql::GraphqlData;

use crate::models::transaction::TransactionBroadcast;

pub(crate) fn map_transaction_broadcast_response(hash: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    if hash.is_empty() { Err("Empty transaction hash".into()) } else { Ok(hash) }
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<GraphqlData<TransactionBroadcast>>(response)?;
    if response.errors.is_some() {
        return Err("Failed to broadcast transaction".into());
    }

    let hash = response
        .data
        .and_then(|data| data.submit_transaction)
        .map(|submit_transaction| submit_transaction.hash)
        .ok_or("Failed to broadcast transaction")?;

    map_transaction_broadcast_response(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_broadcast_response() {
        assert_eq!(map_transaction_broadcast_response("test_hash_123".to_string()).unwrap(), "test_hash_123");
        assert!(map_transaction_broadcast_response(String::new()).is_err());
    }
}
