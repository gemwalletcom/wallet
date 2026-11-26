use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use primitives::{AssetId, EVMChain, swap::SwapQuoteDataType};

use super::{evm::token_transfer, quote_data::QuoteData};
use crate::{Deployment, SwapperError, SwapperQuoteData};

pub(super) fn verify_amounts(from_asset: &AssetId, from_value: &BigUint, data: &SwapperQuoteData) -> Result<(), SwapperError> {
    let swapped_token = match from_asset.token_id.as_deref() {
        Some(token_id) => Some(token_id),
        None => EVMChain::from_chain(from_asset.chain).and_then(|chain| chain.native_token_contract()),
    };
    let is_swapped_token = |token: &str| swapped_token.is_some_and(|swapped| Deployment::contains(from_asset.chain, &[swapped], token));
    let approves_swapped_token = data
        .approval
        .iter()
        .map(|approval| approval.token.as_str())
        .chain(data.permit2.iter().map(|permit2| permit2.token.as_str()))
        .all(is_swapped_token);
    let sends_quoted_value = match (&data.data_type, QuoteData::of(from_asset.chain.chain_type())) {
        (SwapQuoteDataType::Transfer, _) => data.value <= *from_value,
        (SwapQuoteDataType::Contract, QuoteData::EvmCall) => match from_asset.is_native() {
            true => data.value <= *from_value,
            false => data.value == BigUint::ZERO && token_transfer(from_asset, data).is_none_or(|transfer| u256_to_biguint(&transfer.value) <= *from_value),
        },
        (SwapQuoteDataType::Contract, QuoteData::SolanaTransaction | QuoteData::TonMessages | QuoteData::Unread) => true,
    };
    if !approves_swapped_token || !sends_quoted_value {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};
    use alloy_sol_types::SolCall;
    use gem_evm::contracts::IERC20;
    use primitives::{
        Chain,
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_USDT_TOKEN_ID},
        hex,
        swap::{ApprovalData, Permit2ApprovalData},
        testkit::signer_mock::{TEST_BITCOIN_SENDER, TEST_EVM_RECIPIENT},
    };

    #[test]
    fn test_verify_amounts() {
        let coin = AssetId::from_chain(Chain::Ethereum);
        let usdc = AssetId::from_token(Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID);
        let amount = BigUint::from(1_000u64);
        let approval = ApprovalData::make(ETHEREUM_USDC_TOKEN_ID, TEST_EVM_RECIPIENT, amount.clone(), false);
        let permit2 = Permit2ApprovalData {
            token: ETHEREUM_USDC_TOKEN_ID.to_lowercase(),
            ..Permit2ApprovalData::mock()
        };
        let token_call = SwapperQuoteData {
            approval: Some(approval.clone()),
            permit2: Some(permit2.clone()),
            ..SwapperQuoteData::mock()
        };

        assert_eq!(
            verify_amounts(
                &coin,
                &amount,
                &SwapperQuoteData {
                    value: amount.clone(),
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(
            verify_amounts(
                &coin,
                &amount,
                &SwapperQuoteData {
                    value: BigUint::from(999u64),
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(verify_amounts(&usdc, &amount, &token_call), Ok(()));
        let celo_token = EVMChain::Celo.native_token_contract().unwrap();
        assert_eq!(
            verify_amounts(
                &AssetId::from_chain(Chain::Celo),
                &amount,
                &SwapperQuoteData {
                    approval: Some(ApprovalData::make(celo_token, TEST_EVM_RECIPIENT, amount.clone(), false)),
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(()),
            "a coin that is also a token is approved as its own token"
        );
        assert_eq!(
            verify_amounts(
                &AssetId::from_chain(Chain::Solana),
                &amount,
                &SwapperQuoteData {
                    value: amount.clone() + 1u32,
                    ..SwapperQuoteData::mock()
                }
            ),
            Ok(())
        );
        assert_eq!(
            verify_amounts(&AssetId::from_chain(Chain::Bitcoin), &amount, &SwapperQuoteData::new_transfer(TEST_BITCOIN_SENDER.to_string(), amount.clone(), None)),
            Ok(())
        );

        for (asset, data, reason) in [
            (
                &coin,
                SwapperQuoteData {
                    value: amount.clone() + 1u32,
                    ..SwapperQuoteData::mock()
                },
                "a coin swap that sends more than quoted",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    value: BigUint::from(1u32),
                    ..token_call.clone()
                },
                "a token swap that also sends coins",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    approval: Some(ApprovalData::make(ETHEREUM_USDT_TOKEN_ID, TEST_EVM_RECIPIENT, amount.clone(), false)),
                    ..token_call.clone()
                },
                "an approval for another token",
            ),
            (
                &usdc,
                SwapperQuoteData {
                    permit2: Some(Permit2ApprovalData {
                        token: ETHEREUM_USDT_TOKEN_ID.to_string(),
                        ..permit2
                    }),
                    ..token_call
                },
                "a Permit2 signature for another token",
            ),
            (
                &coin,
                SwapperQuoteData {
                    approval: Some(approval),
                    ..SwapperQuoteData::mock()
                },
                "an approval on a coin swap",
            ),
            (
                &AssetId::from_chain(Chain::Bitcoin),
                SwapperQuoteData::new_transfer(TEST_BITCOIN_SENDER.to_string(), amount.clone() + 1u32, None),
                "a deposit that sends more than quoted",
            ),
            (
                &usdc,
                SwapperQuoteData::new_contract(
                    ETHEREUM_USDC_TOKEN_ID.to_string(),
                    BigUint::ZERO,
                    hex::encode(
                        IERC20::transferCall {
                            to: Address::ZERO,
                            value: U256::from(1_001u64),
                        }
                        .abi_encode(),
                    ),
                    None,
                    None,
                ),
                "a token transfer that sends more than quoted",
            ),
        ] {
            assert_eq!(verify_amounts(asset, &amount, &data), Err(SwapperError::InvalidRoute), "{reason}");
        }
    }
}
