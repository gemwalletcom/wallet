use std::collections::HashMap;

use num_bigint::{BigInt, BigUint};
use primitives::Address as _;

use crate::address::TronAddress;
use crate::models::{InternalTransaction, TronLog};
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;

pub fn token_balance_deltas(logs: &[TronLog], owner: &TronAddress) -> HashMap<String, BigInt> {
    let mut deltas: HashMap<String, BigInt> = HashMap::new();
    for log in logs {
        let Some((token, delta)) = decode_transfer_delta(log, owner) else { continue };
        *deltas.entry(token).or_default() += delta;
    }
    deltas
}

pub fn internal_transaction_deltas(internal_transactions: &[InternalTransaction], owner: &TronAddress) -> HashMap<Option<String>, BigInt> {
    let mut deltas: HashMap<Option<String>, BigInt> = HashMap::new();
    for internal in internal_transactions {
        if internal.rejected {
            continue;
        }
        let Some(caller) = internal.caller_address else { continue };
        let Some(recipient) = internal.transfer_to_address else { continue };
        if caller == recipient || (caller != *owner && recipient != *owner) {
            continue;
        }
        let is_incoming = recipient == *owner;

        for call_value in &internal.call_value_info {
            if call_value.call_value == 0 {
                continue;
            }
            let amount = BigInt::from(call_value.call_value);
            *deltas.entry(call_value.token_id.clone()).or_default() += if is_incoming { amount } else { -amount };
        }
    }
    deltas
}

fn decode_transfer_delta(log: &TronLog, owner: &TronAddress) -> Option<(String, BigInt)> {
    let (token, from, to, amount) = decode_token_transfer(log)?;
    let amount = BigInt::from(amount);

    let delta = match (from == *owner, to == *owner) {
        (false, false) => return None,
        (true, false) => -amount,
        (false, true) => amount,
        (true, true) => BigInt::default(),
    };

    Some((token?.encode(), delta))
}

pub(super) fn decode_token_transfer(log: &TronLog) -> Option<(Option<TronAddress>, TronAddress, TronAddress, BigUint)> {
    let topics = log.topics.as_ref()?;
    if topics.len() != 3 || topics[0] != ERC20_TRANSFER_EVENT_SIGNATURE {
        return None;
    }

    let from = TronAddress::from_topic(&topics[1])?;
    let to = TronAddress::from_topic(&topics[2])?;
    let amount = BigUint::parse_bytes(log.data.as_deref()?.as_bytes(), 16)?;

    Some((log.address, from, to, amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_owner() -> TronAddress {
        TronAddress::from_hex_or_base58("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM").unwrap()
    }

    #[test]
    fn test_decode_transfer_delta_ignores_uninvolved_transfer() {
        let log = TronLog::mock_transfer(
            "DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss",
            "0000000000000000000000000344a87b2c5bc1cd9407fb9bd0c325a4403af30b",
            "0000000000000000000000004e4bee11cea0070f957b98fd8cf4138ef3295e0e",
            "00000000000000000000000000000000000000000000000000000000000f4240",
        );

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

    #[test]
    fn test_internal_transaction_deltas_credits_native_trx_received_by_owner() {
        let internal = InternalTransaction::mock("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss", "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM", 1_000_000, None, false);

        let deltas = internal_transaction_deltas(&[internal], &mock_owner());

        assert_eq!(deltas.get(&None), Some(&BigInt::from(1_000_000)));
    }

    #[test]
    fn test_internal_transaction_deltas_ignores_rejected_and_uninvolved_and_self_transfers() {
        let rejected = InternalTransaction::mock("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss", "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM", 1_000_000, None, true);
        let uninvolved = InternalTransaction::mock("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss", "TH7CfjAfb2WxLSSGX4w5iziCj42qK8S36Y", 1_000_000, None, false);
        let self_transfer = InternalTransaction::mock("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM", "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM", 1_000_000, None, false);

        let deltas = internal_transaction_deltas(&[rejected, uninvolved, self_transfer], &mock_owner());

        assert!(deltas.is_empty());
    }
}
