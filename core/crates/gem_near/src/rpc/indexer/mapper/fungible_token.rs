use std::error::Error;

use num_bigint::BigUint;
use primitives::transaction_metadata_types::TransactionAssetTransfer;
use primitives::{AssetId, Chain};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::deserialize_biguint_from_str;

use crate::constants::{EVENT_JSON_PREFIX, FUNGIBLE_TOKEN_TRANSFER_EVENT, NEP_141_STANDARD};

use super::super::model::FastNearReceipt;

#[derive(Debug, Deserialize)]
struct FastNearEvent {
    standard: String,
    event: String,
    data: Value,
}

#[derive(Debug, Deserialize)]
struct FastNearFungibleTokenTransfer {
    old_owner_id: String,
    new_owner_id: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount: BigUint,
}

pub(super) fn map_fungible_token_transfers(receipts: &[FastNearReceipt]) -> Result<Vec<TransactionAssetTransfer>, Box<dyn Error + Send + Sync>> {
    let mut asset_transfers = Vec::<TransactionAssetTransfer>::new();
    for receipt in receipts {
        for log in &receipt.execution_outcome.outcome.logs {
            let Some(transfers) = parse_fungible_token_transfer_event(log)? else {
                continue;
            };
            let contract = &receipt.receipt.receiver_id;
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

fn parse_fungible_token_transfer_event(log: &str) -> Result<Option<Vec<FastNearFungibleTokenTransfer>>, Box<dyn Error + Send + Sync>> {
    let Some(event_json) = log.strip_prefix(EVENT_JSON_PREFIX) else {
        return Ok(None);
    };
    let event: FastNearEvent = serde_json::from_str(event_json)?;
    if event.standard != NEP_141_STANDARD || event.event != FUNGIBLE_TOKEN_TRANSFER_EVENT {
        return Ok(None);
    }
    Ok(Some(serde_json::from_value(event.data)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_other_event_data_shapes() {
        let log = format!(r#"{EVENT_JSON_PREFIX}{{"standard":"potlock","event":"donation","data":{{"amount":"1"}}}}"#);

        assert!(parse_fungible_token_transfer_event(&log).unwrap().is_none());
    }
}
