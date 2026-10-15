use std::error::Error;

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use primitives::swap::SwapQuoteDataType;
use primitives::{FeePriority, FeeRate, GasPriceType, StakeType, TransactionInputType, TransactionLoadInput};

use crate::{
    encode_split_and_stake, encode_token_transfer, encode_transfer, encode_unstake,
    models::{Coin, Gas, Object, OwnedCoins, StakeInput, SuiObject, TokenTransferInput, TransferInput, TxOutput, UnstakeInput},
    validate_and_hash,
};

pub fn map_transaction_fee_rates(base_gas_price: BigInt) -> Vec<FeeRate> {
    vec![
        FeeRate::new(FeePriority::Normal, GasPriceType::regular(&base_gas_price * BigInt::from(110) / BigInt::from(100))),
        FeeRate::new(FeePriority::Fast, GasPriceType::regular(&base_gas_price * BigInt::from(2))),
    ]
}

pub fn map_transaction_data(input: TransactionLoadInput, sui_coins: OwnedCoins<Coin>, token_coins: Option<OwnedCoins<Coin>>, objects: Vec<SuiObject>, gas_budget: u64) -> Result<String, Box<dyn Error + Send + Sync>> {
    let gas_price = input.gas_price.gas_price_u64()?;

    let tx_output = match input.input_type {
        TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } | TransactionInputType::Deposit { asset } => encode_asset_transfer(
            TransferInput {
                sender: input.sender_address,
                recipient: input.destination_address,
                amount: input.value.to_u64().ok_or("Sui amount is too large")?,
                coins: sui_coins,
                send_max: input.is_max_value,
                gas: Gas { budget: gas_budget, price: gas_price },
            },
            asset.id.token_id.as_deref(),
            token_coins,
        ),
        TransactionInputType::Stake { stake_type, .. } => match stake_type {
            StakeType::Stake(validator) => {
                let stake_input = StakeInput {
                    sender: input.sender_address,
                    validator: validator.id,
                    stake_amount: input.value.to_u64().ok_or("Sui amount is too large")?,
                    gas: Gas { budget: gas_budget, price: gas_price },
                    coins: sui_coins,
                };
                encode_split_and_stake(&stake_input)
            }
            StakeType::Unstake(delegation) => {
                let gas_coin = sui_coins.coins.first().ok_or("No gas coins available for unstake")?.clone();
                let staked_object = objects.iter().find(|obj| obj.object_id == delegation.base.delegation_id).ok_or("Staked SUI object not found in provided objects")?;

                let staked_sui = Object {
                    object_id: staked_object.object_id.parse().map_err(|error| format!("invalid staked Sui object id: {error}"))?,
                    version: staked_object.version.parse().map_err(|error| format!("invalid staked Sui object version: {error}"))?,
                    digest: staked_object.digest.parse().map_err(|error| format!("invalid staked Sui object digest: {error}"))?,
                };

                let unstake_input = UnstakeInput {
                    sender: input.sender_address,
                    staked_sui,
                    gas: Gas { budget: gas_budget, price: gas_price },
                    gas_coin,
                };
                encode_unstake(&unstake_input)
            }
            StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => Err("Unsupported stake type for Sui".into()),
        },
        TransactionInputType::Swap { from_asset, swap_data, .. } => match swap_data.data.data_type {
            SwapQuoteDataType::Transfer => encode_asset_transfer(
                TransferInput {
                    sender: input.sender_address,
                    recipient: swap_data.data.to,
                    amount: swap_data.data.value.to_u64().ok_or("Sui amount is too large")?,
                    coins: sui_coins,
                    send_max: false,
                    gas: Gas { budget: gas_budget, price: gas_price },
                },
                from_asset.id.token_id.as_deref(),
                token_coins,
            ),
            SwapQuoteDataType::Contract => validate_and_hash(&swap_data.data.data),
        },
        TransactionInputType::Generic { extra, .. } | TransactionInputType::Payment { extra, .. } => {
            let raw_data = extra.data.ok_or("Missing transaction data for Sui generic input")?;
            let encoded = String::from_utf8(raw_data).map_err(|_| "Invalid UTF-8 data for Sui generic input")?;
            validate_and_hash(&encoded)
        }
        TransactionInputType::TokenApprove { .. } | TransactionInputType::TransferNft { .. } | TransactionInputType::Account { .. } | TransactionInputType::Perpetual { .. } | TransactionInputType::Earn { .. } => {
            Err("Unsupported transaction type for Sui".into())
        }
    }?;
    Ok(format!("{}_{}", tx_output.base64_encoded(), hex::encode(tx_output.hash)))
}

fn encode_asset_transfer(input: TransferInput, token_id: Option<&str>, token_coins: Option<OwnedCoins<Coin>>) -> Result<TxOutput, Box<dyn Error + Send + Sync>> {
    match token_id {
        None => encode_transfer(&input),
        Some(_) => {
            let gas_coin = input.coins.coins.into_iter().next().ok_or("No gas coins available for token transfer")?;
            let tokens = token_coins.ok_or("Missing token coins set for Sui token transfer")?;
            encode_token_transfer(&TokenTransferInput {
                sender: input.sender,
                recipient: input.recipient,
                amount: input.amount,
                tokens,
                gas: input.gas,
                gas_coin,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TEST_ADDRESS_EMPTY;
    use num_bigint::BigUint;
    use primitives::{Asset, AssetType, Chain, Delegation, StakeType, SwapProvider, TransactionInputType, TransactionLoadInput, asset_constants::SUI_USDC_ASSET_ID, swap::SwapData};

    #[test]
    fn test_swap_transfer_uses_deposit_amount_and_recipient() {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap {
                from_asset: Asset::from_chain(Chain::Sui),
                to_asset: Asset::from_chain(Chain::Ton),
                swap_data: SwapData::mock_transfer(SwapProvider::SwapsXyz, "2000000000", "1", TEST_ADDRESS_EMPTY),
            },
            is_max_value: true,
            ..TransactionLoadInput::mock()
        };
        let normal_transfer = TransactionLoadInput {
            destination_address: TEST_ADDRESS_EMPTY.to_string(),
            value: BigUint::from(2_000_000_000_u64),
            ..TransactionLoadInput::mock()
        };
        assert_eq!(
            map_transaction_data(input, OwnedCoins::mock_sui(), None, vec![], 1_000_000).unwrap(),
            map_transaction_data(normal_transfer, OwnedCoins::mock_sui(), None, vec![], 1_000_000).unwrap()
        );
    }

    #[test]
    fn test_token_swap_transfer_uses_deposit_amount_and_recipient() {
        let asset = Asset::new(SUI_USDC_ASSET_ID.clone(), "USDC".into(), "USDC".into(), 6, AssetType::TOKEN);
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap {
                from_asset: asset.clone(),
                to_asset: Asset::from_chain(Chain::Ton),
                swap_data: SwapData::mock_transfer(SwapProvider::NearIntents, "10000000", "1", TEST_ADDRESS_EMPTY),
            },
            is_max_value: true,
            ..TransactionLoadInput::mock()
        };
        let normal_transfer = TransactionLoadInput {
            input_type: TransactionInputType::Transfer { asset: asset.clone() },
            destination_address: TEST_ADDRESS_EMPTY.to_string(),
            value: BigUint::from(10_000_000_u64),
            ..TransactionLoadInput::mock()
        };
        let token_coins = OwnedCoins::new(asset.id.token_id.unwrap(), vec![], 20_000_000);
        assert_eq!(
            map_transaction_data(input, OwnedCoins::mock_sui(), Some(token_coins.clone()), vec![], 1_000_000).unwrap(),
            map_transaction_data(normal_transfer, OwnedCoins::mock_sui(), Some(token_coins), vec![], 1_000_000).unwrap()
        );
    }

    #[test]
    fn test_contract_swap_preserves_payload() {
        let message = map_transaction_data(TransactionLoadInput::mock(), OwnedCoins::mock_sui(), None, vec![], 25_000_000).unwrap();
        let (payload, _) = message.split_once('_').unwrap();
        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Sui),
            to_asset: Asset::from_chain(Chain::Sui),
            swap_data: SwapData::mock_with_provider_data(SwapProvider::CetusClmm, payload, None),
        });
        assert_eq!(map_transaction_data(input, OwnedCoins::default(), None, vec![], 1_000_000).unwrap(), message);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Sui),
            to_asset: Asset::from_chain(Chain::Sui),
            swap_data: SwapData::mock_with_provider_data(SwapProvider::CetusClmm, "invalid transaction", None),
        });
        assert!(map_transaction_data(input, OwnedCoins::mock_sui(), None, vec![], 1_000_000).is_err());
    }

    #[test]
    fn test_unstake_uses_object_reference() {
        let delegation_id = "0x1234567890abcdef1234567890abcdef12345678";
        let delegation = Delegation::mock_with_id(delegation_id.to_string());
        let input_type = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Sui),
            stake_type: StakeType::Unstake(delegation),
        };
        let input = TransactionLoadInput::mock_with_input_type(input_type);

        let gas_coins = OwnedCoins::<Coin>::mock_sui();

        let objects = vec![SuiObject {
            object_id: delegation_id.to_string(),
            version: "12345".to_string(),
            digest: "CU86BjXRF1XHFRjKBasCYEuaQxhHuyGBpuoJyqsrYoX5".to_string(),
        }];

        assert!(map_transaction_data(input.clone(), gas_coins.clone(), None, objects.clone(), 25_000_000).is_ok());
        let invalid_objects = vec![SuiObject {
            version: "invalid".to_string(),
            ..objects[0].clone()
        }];
        assert!(map_transaction_data(input, gas_coins, None, invalid_objects, 25_000_000).is_err());
    }

    #[test]
    fn test_map_transaction_fee_rates() {
        let rates = map_transaction_fee_rates(BigInt::from(497));

        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].priority, FeePriority::Normal);
        assert_eq!(rates[0].gas_price_type.gas_price(), BigInt::from(546));
        assert_eq!(rates[1].priority, FeePriority::Fast);
        assert_eq!(rates[1].gas_price_type.gas_price(), BigInt::from(994));
    }
}
