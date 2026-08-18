use std::{collections::HashMap, error::Error};

use num_bigint::BigInt;
use primitives::{FeeOption, TransactionFee, TransactionLoadInput, TransactionLoadMetadata};

use crate::{
    address::is_implicit_address,
    constants::FUNGIBLE_TOKEN_FUNCTION_CALL_GAS,
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

pub(super) fn map_transaction_fee(
    input: &TransactionLoadInput,
    destination_address: &str,
    config: &ProtocolConfig,
    token_account_creation_deposit: Option<BigInt>,
) -> TransactionFee {
    let costs = &config.runtime_config.transaction_costs;
    let action_costs = &costs.action_creation_config;

    let asset_id = &input.input_type.get_asset().id;
    let (send_gas, execution_gas, options) = if asset_id.is_token() {
        let sender_is_receiver = asset_id.token_id.as_deref() == Some(input.sender_address.as_str());
        let function_call_count = if token_account_creation_deposit.is_some() { 2u32 } else { 1u32 };
        let send_gas = BigInt::from(costs.action_receipt_creation_config.send_gas(sender_is_receiver))
            + BigInt::from(action_costs.function_call_cost.send_gas(sender_is_receiver)) * function_call_count;
        let execution_gas = BigInt::from(costs.action_receipt_creation_config.execution)
            + BigInt::from(action_costs.function_call_cost.execution) * function_call_count
            + BigInt::from(FUNGIBLE_TOKEN_FUNCTION_CALL_GAS) * function_call_count;
        let options = token_account_creation_deposit
            .map(|value| HashMap::from([(FeeOption::TokenAccountCreation, value)]))
            .unwrap_or_default();
        (send_gas, execution_gas, options)
    } else {
        let sender_is_receiver = input.sender_address == destination_address;
        let mut send_gas = BigInt::from(costs.action_receipt_creation_config.send_gas(sender_is_receiver)) + BigInt::from(action_costs.transfer_cost.send_gas(sender_is_receiver));
        let mut execution_gas = BigInt::from(costs.action_receipt_creation_config.execution) + BigInt::from(action_costs.transfer_cost.execution);

        if is_implicit_address(destination_address) {
            send_gas += BigInt::from(action_costs.create_account_cost.send_gas(sender_is_receiver));
            send_gas += BigInt::from(action_costs.add_key_cost.full_access_cost.send_gas(sender_is_receiver));
            execution_gas += BigInt::from(action_costs.create_account_cost.execution);
            execution_gas += BigInt::from(action_costs.add_key_cost.full_access_cost.execution);
        }

        (send_gas, execution_gas, HashMap::new())
    };

    let current_gas_price = input.gas_price.gas_price();
    let receipt_gas_price = current_gas_price.clone().max(BigInt::from(config.runtime_config.min_gas_purchase_price));
    let fee = &send_gas * &current_gas_price + &execution_gas * receipt_gas_price;
    let gas_limit = &send_gas + &execution_gas;

    TransactionFee::new_gas_price_type(input.gas_price.clone(), fee, gas_limit, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountAccessKey, Block, BlockHeader};
    use primitives::{Asset, Chain, GasPriceType, SwapProvider, TransactionInputType, swap::SwapData};

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

        let implicit_fee = map_transaction_fee(&input, &input.destination_address, &config, None);
        assert_eq!(implicit_fee.fee, BigInt::from(7607442456250000000000u128));

        input.input_type = TransactionInputType::Swap(
            Asset::from_chain(Chain::Near),
            Asset::from_chain(Chain::Near),
            SwapData::mock_transfer(SwapProvider::NearIntents, "1", "1", "051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95"),
        );
        input.destination_address = input.sender_address.clone();
        let swap_destination = input.input_type.swap_to_address().unwrap();
        let swap_fee = map_transaction_fee(&input, swap_destination, &config, None);
        assert_eq!(swap_fee.fee, implicit_fee.fee);
        assert_eq!(swap_fee.gas_limit, implicit_fee.gas_limit);

        input.input_type = TransactionInputType::Transfer(Asset::from_chain(Chain::Near));
        input.destination_address = "receiver.near".to_string();
        let named_fee = map_transaction_fee(&input, &input.destination_address, &config, None);
        assert_eq!(named_fee.fee, BigInt::from(245500818750000000000u128));

        input.input_type = primitives::TransactionInputType::Transfer(primitives::Asset::new(
            primitives::AssetId::from_token(primitives::Chain::Near, "token.near"),
            "Token".to_string(),
            "TKN".to_string(),
            6,
            primitives::AssetType::TOKEN,
        ));
        let token_fee = map_transaction_fee(&input, &input.destination_address, &config, None);
        assert_eq!(token_fee.gas_limit, BigInt::from(31_196_119_000_000u64));
        assert_eq!(token_fee.fee, BigInt::from(30_918_865_450_000_000_000_000u128));

        let token_account_creation = BigInt::from(1_250_000_000_000_000_000_000u128);
        let token_fee = map_transaction_fee(&input, &input.destination_address, &config, Some(token_account_creation.clone()));
        assert_eq!(token_fee.gas_limit, BigInt::from(62_176_119_000_000u64));
        assert_eq!(token_fee.options.get(&FeeOption::TokenAccountCreation), Some(&token_account_creation));
        assert_eq!(token_fee.fee, BigInt::from(62_968_865_450_000_000_000_000u128));
    }
}
