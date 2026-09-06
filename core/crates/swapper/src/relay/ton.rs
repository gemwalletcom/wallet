use gem_encoding::decode_base64;
use gem_ton::{Address, models::TraceResponse, rpc::client::TonClient};

use crate::{RpcClient, SwapperError};

pub struct TonDeposit {
    pub sender: String,
    pub transaction_hashes: Vec<Vec<u8>>,
}

pub async fn find_deposit(client: &TonClient<RpcClient>, transaction_hash: &str) -> Result<Option<TonDeposit>, SwapperError> {
    let traces = client.get_traces_by_hash(transaction_hash.to_string()).await.map_err(SwapperError::transaction_error)?;
    Ok(map_deposit(&traces))
}

pub fn map_deposit(traces: &TraceResponse) -> Option<TonDeposit> {
    let trace = traces.traces.first()?;
    let root = trace.root_transaction()?;
    let sender = Address::try_parse_hex(&root.in_msg.as_ref()?.destination)?.encode_non_bounceable();
    let transaction_hashes = trace
        .transactions
        .values()
        .filter(|transaction| transaction.hash != root.hash)
        .filter_map(|transaction| decode_base64(&transaction.hash).ok())
        .collect();
    Some(TonDeposit { sender, transaction_hashes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::RelayRequestsResponse;
    use primitives::decode_hex;

    #[test]
    fn test_map_deposit() {
        let traces: TraceResponse = serde_json::from_str(include_str!("testdata/ton_deposit_trace.json")).unwrap();
        let deposit = map_deposit(&traces).unwrap();
        let root_hash = decode_hex("e86159ff0662a587649bc1d2ff0cd146e6628c3cc37396f7b680bd28260f44b5").unwrap();
        let depository_hash = decode_hex("e7844ac5fd48b3f5dcfbcecb34a8d5cfe2614ea6c4e4dc1cbd5b528db6ab5fac").unwrap();

        assert_eq!(deposit.sender, "UQD7kvf5vlcNnXumsVR86RfZgLKcG3e2jLZymf2A4-ruvbii");
        assert_eq!(deposit.transaction_hashes.len(), 3);
        assert!(deposit.transaction_hashes.contains(&depository_hash));
        assert!(!deposit.transaction_hashes.contains(&root_hash));

        let response: RelayRequestsResponse = serde_json::from_str(include_str!("testdata/request_ton_to_robinhood.json")).unwrap();
        assert!(response.requests[0].has_input_transaction(&deposit.transaction_hashes));

        assert!(map_deposit(&TraceResponse { traces: vec![] }).is_none());
    }
}
