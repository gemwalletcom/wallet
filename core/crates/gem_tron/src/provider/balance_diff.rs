use std::collections::HashMap;

use num_bigint::BigInt;
use primitives::Address as _;

use crate::address::TronAddress;
use crate::models::TronLog;
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;

pub fn token_balance_deltas(logs: &[TronLog], owner: &TronAddress) -> HashMap<String, BigInt> {
    let mut deltas: HashMap<String, BigInt> = HashMap::new();
    for log in logs {
        let Some((token, delta)) = decode_transfer_delta(log, owner) else { continue };
        *deltas.entry(token).or_default() += delta;
    }
    deltas
}

fn decode_transfer_delta(log: &TronLog, owner: &TronAddress) -> Option<(String, BigInt)> {
    let topics = log.topics.as_ref()?;
    if topics.len() != 3 || topics[0] != ERC20_TRANSFER_EVENT_SIGNATURE {
        return None;
    }

    let from = TronAddress::from_topic(&topics[1])?;
    let to = TronAddress::from_topic(&topics[2])?;
    let amount = BigInt::parse_bytes(log.data.as_deref()?.as_bytes(), 16)?;

    let delta = match (from == *owner, to == *owner) {
        (false, false) => return None,
        (true, false) => -amount,
        (false, true) => amount,
        (true, true) => BigInt::default(),
    };

    Some((log.address?.encode(), delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_owner() -> TronAddress {
        TronAddress::from_hex_or_base58("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM").unwrap()
    }

    #[test]
    fn test_decode_transfer_delta_ignores_uninvolved_transfer() {
        let log = TronLog {
            address: TronAddress::from_hex_or_base58("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss"),
            topics: Some(vec![
                ERC20_TRANSFER_EVENT_SIGNATURE.to_string(),
                "0000000000000000000000000344a87b2c5bc1cd9407fb9bd0c325a4403af30b".to_string(),
                "0000000000000000000000004e4bee11cea0070f957b98fd8cf4138ef3295e0e".to_string(),
            ]),
            data: Some("00000000000000000000000000000000000000000000000000000000000f4240".to_string()),
        };

        assert!(decode_transfer_delta(&log, &mock_owner()).is_none());
    }

    #[test]
    fn test_decode_transfer_delta_ignores_non_transfer_topic() {
        let log = TronLog {
            address: TronAddress::from_hex_or_base58("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss"),
            topics: Some(vec!["e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c".to_string()]),
            data: Some("00".to_string()),
        };

        assert!(decode_transfer_delta(&log, &mock_owner()).is_none());
    }
}
