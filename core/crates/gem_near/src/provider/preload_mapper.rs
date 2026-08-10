use std::{collections::HashMap, error::Error};

use num_bigint::BigInt;
use primitives::{TransactionFee, TransactionLoadInput, TransactionLoadMetadata};

use crate::{
    address::is_implicit_address,
    models::{AccountAccessKey, Block, ProtocolConfig},
};

pub fn address_to_public_key(address: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let address_bytes = hex::decode(address)?;
    let encoded = bs58::encode(address_bytes).into_string();
    Ok(format!("ed25519:{}", encoded))
}

pub fn map_transaction_preload(access_key: &AccountAccessKey, block: &Block) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Near {
        sequence: (access_key.nonce + 1) as u64,
        block_hash: block.header.hash.clone(),
    }
}

pub(super) fn map_transaction_fee(input: &TransactionLoadInput, config: &ProtocolConfig) -> TransactionFee {
    let sender_is_receiver = input.sender_address == input.destination_address;
    let costs = &config.runtime_config.transaction_costs;
    let action_costs = &costs.action_creation_config;

    let mut send_gas = BigInt::from(costs.action_receipt_creation_config.send_gas(sender_is_receiver)) + BigInt::from(action_costs.transfer_cost.send_gas(sender_is_receiver));
    let mut execution_gas = BigInt::from(costs.action_receipt_creation_config.execution) + BigInt::from(action_costs.transfer_cost.execution);

    if is_implicit_address(&input.destination_address) {
        send_gas += BigInt::from(action_costs.create_account_cost.send_gas(sender_is_receiver));
        send_gas += BigInt::from(action_costs.add_key_cost.full_access_cost.send_gas(sender_is_receiver));
        execution_gas += BigInt::from(action_costs.create_account_cost.execution);
        execution_gas += BigInt::from(action_costs.add_key_cost.full_access_cost.execution);
    }

    let current_gas_price = input.gas_price.gas_price();
    let receipt_gas_price = current_gas_price.clone().max(BigInt::from(config.runtime_config.min_gas_purchase_price));
    let fee = &send_gas * &current_gas_price + &execution_gas * receipt_gas_price;
    let gas_limit = send_gas + execution_gas;

    TransactionFee::new_gas_price_type(input.gas_price.clone(), fee, gas_limit, HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountAccessKey, Block, BlockHeader};
    use primitives::GasPriceType;

    #[test]
    fn test_address_to_public_key() {
        let address = "051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95";
        let result = address_to_public_key(address).unwrap();
        assert!(result.starts_with("ed25519:"));
    }

    #[test]
    fn test_map_transaction_preload() {
        let access_key = AccountAccessKey { nonce: 116479371000026 };

        let block = Block {
            header: BlockHeader {
                hash: "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz".to_string(),
                height: 12345,
            },
        };

        let result = map_transaction_preload(&access_key, &block);

        match result {
            TransactionLoadMetadata::Near { sequence, block_hash } => {
                assert_eq!(sequence, 116479371000027);
                assert_eq!(block_hash, "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz");
            }
            _ => panic!("Expected Near metadata"),
        }
    }

    #[test]
    fn test_map_transaction_fee() {
        let config: ProtocolConfig = serde_json::from_str(include_str!("../../testdata/protocol_config.json")).unwrap();
        let mut input = TransactionLoadInput::mock_near(
            "sender.near",
            "051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95",
            "1",
            1,
            "244ZQ9cgj3CQ6bWBdytfrJMuMQ1jdXLFGnr4HhvtCTnM",
        );
        input.gas_price = GasPriceType::regular(BigInt::from(100000000u64));

        let implicit_fee = map_transaction_fee(&input, &config);
        assert_eq!(implicit_fee.fee, BigInt::from(7607442456250000000000u128));

        input.destination_address = "receiver.near".to_string();
        let named_fee = map_transaction_fee(&input, &config);
        assert_eq!(named_fee.fee, BigInt::from(245500818750000000000u128));
    }
}
