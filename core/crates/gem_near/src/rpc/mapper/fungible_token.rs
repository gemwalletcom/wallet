use std::error::Error;

use num_bigint::BigUint;
use primitives::transaction_metadata_types::TransactionAssetTransfer;
use primitives::{AssetId, Chain};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::deserialize_biguint_from_str;

use crate::{
    constants::{EVENT_JSON_PREFIX, FUNGIBLE_TOKEN_TRANSFER_EVENT, NEP_141_STANDARD},
    models::Outcome,
};

#[derive(Debug, Deserialize)]
struct NearEvent {
    standard: Option<String>,
    event: Option<String>,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct FungibleTokenTransfer {
    old_owner_id: String,
    new_owner_id: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount: BigUint,
}

pub(super) fn map_fungible_token_transfers(receipts: &[Outcome]) -> Result<Vec<TransactionAssetTransfer>, Box<dyn Error + Send + Sync>> {
    let mut asset_transfers = Vec::<TransactionAssetTransfer>::new();
    for receipt in receipts {
        for log in &receipt.logs {
            let Some(transfers) = parse_fungible_token_transfer_event(log) else {
                continue;
            };
            let contract = receipt.executor_id.as_deref().ok_or("missing NEAR receipt executor id")?;
            if !crate::address::is_valid_account_id(contract) {
                return Err(format!("invalid NEP-141 contract id: {contract}").into());
            }
            let asset_id = AssetId::from_token(Chain::Near, contract);
            for transfer in transfers {
                let transfer = TransactionAssetTransfer {
                    asset_id: asset_id.clone(),
                    from: transfer.old_owner_id,
                    to: transfer.new_owner_id,
                    value: transfer.amount,
                };
                match asset_transfers
                    .iter_mut()
                    .find(|existing| existing.asset_id == transfer.asset_id && existing.from == transfer.from && existing.to == transfer.to)
                {
                    Some(existing) => existing.value += transfer.value,
                    None => asset_transfers.push(transfer),
                }
            }
        }
    }
    Ok(asset_transfers)
}

fn parse_fungible_token_transfer_event(log: &str) -> Option<Vec<FungibleTokenTransfer>> {
    let event: NearEvent = serde_json::from_str(log.strip_prefix(EVENT_JSON_PREFIX)?).ok()?;
    if event.standard.as_deref() != Some(NEP_141_STANDARD) || event.event.as_deref() != Some(FUNGIBLE_TOKEN_TRANSFER_EVENT) {
        return None;
    }
    serde_json::from_value(event.data?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fungible_token_transfer_event() {
        for event in [
            r#"{"event":"donation","data":{"amount":"1"}}"#,
            r#"{"standard":"potlock","event":"donation","data":{"amount":"1"}}"#,
            "not json",
            r#"{"standard":"nep141","event":"ft_transfer"}"#,
            r#"{"standard":"nep141","event":"ft_transfer","data":[{"old_owner_id":"a.near","new_owner_id":"b.near","amount":"one"}]}"#,
        ] {
            assert!(parse_fungible_token_transfer_event(&format!("{EVENT_JSON_PREFIX}{event}")).is_none());
        }
    }

    #[test]
    fn test_map_fungible_token_transfers() {
        let receipt: Outcome = serde_json::from_value(serde_json::json!({
            "executor_id": "token.near",
            "logs": [format!("{EVENT_JSON_PREFIX}not json"), format!(r#"{EVENT_JSON_PREFIX}{{"standard":"nep141","event":"ft_transfer","data":[{{"old_owner_id":"a.near","new_owner_id":"b.near","amount":"5"}}]}}"#)],
            "status": {"SuccessValue": ""},
            "tokens_burnt": "0"
        }))
        .unwrap();

        let transfers = map_fungible_token_transfers(&[receipt]).unwrap();

        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].value, BigUint::from(5u32));
    }
}
