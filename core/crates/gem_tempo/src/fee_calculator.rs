use std::error::Error;

use alloy_primitives::Address;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload;
use gem_evm::rpc::{EthereumClient, EvmFeeCalculator};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, TransactionFee, TransactionLoadInput, TransactionType, asset_constants::TEMPO_PATHUSD_ASSET_ID};

use crate::contracts::{ITIP20, ITempoFeeManager};
use crate::fee::{FEE_MANAGER_ADDRESS, USD_CURRENCY, decode_set_user_fee_token, scale_fee_to_token_units};

pub(crate) struct TempoFeeCalculator<C: Client + Clone> {
    client: EthereumClient<C>,
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for TempoFeeCalculator<C> {
    async fn calculate_fee(&self, input: &TransactionLoadInput, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let fee_asset = self.fee_asset(input).await?;
        let transaction_fee = preload::calculate_fee(input, gas_limit)?;

        Ok(TransactionFee {
            fee: scale_fee_to_token_units(transaction_fee.fee),
            fee_asset,
            ..transaction_fee
        })
    }
}

impl<C: Client + Clone> TempoFeeCalculator<C> {
    pub(crate) fn new(client: EthereumClient<C>) -> Self {
        Self { client }
    }

    async fn fee_asset(&self, input: &TransactionLoadInput) -> Result<AssetId, Box<dyn Error + Sync + Send>> {
        let input_type = &input.input_type;
        if let Some(token) = decode_set_user_fee_token(input_type) {
            return self.validate_fee_token(token).await;
        }

        if input_type.transaction_type() != TransactionType::Swap {
            let account_fee_token = self.user_fee_token(&input.sender_address).await?;
            if !account_fee_token.is_zero() {
                return self.validate_fee_token(account_fee_token).await;
            }
        }

        let fee_asset = input_type.get_asset().id.clone();
        let token_id = fee_asset.get_token_id()?;
        if self.tip20_currency(token_id).await? != USD_CURRENCY {
            return Ok(TEMPO_PATHUSD_ASSET_ID.clone());
        }

        Ok(fee_asset)
    }

    async fn validate_fee_token(&self, token: Address) -> Result<AssetId, Box<dyn Error + Sync + Send>> {
        let token_id = token.to_checksum(None);
        if self.tip20_currency(&token_id).await? != USD_CURRENCY {
            return Err("Tempo fee token must use USD currency".into());
        }
        Ok(AssetId::from_token(Chain::Tempo, &token_id))
    }

    async fn user_fee_token(&self, address: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        self.client
            .call_contract(FEE_MANAGER_ADDRESS.parse()?, ITempoFeeManager::userTokensCall { user: address.parse()? })
            .await
    }

    async fn tip20_currency(&self, token_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.client.call_contract(token_id.parse()?, ITIP20::currencyCall {}).await
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, hex::encode_prefixed};
    use alloy_sol_types::SolCall;
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use gem_evm::constants::TOKEN_TRANSFER_GAS_LIMIT;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{
        Asset, AssetType, Chain, EVMChain, GasPriceType, SwapProvider, TransactionInputType, TransactionLoadInput,
        asset_constants::{TEMPO_PATHUSD_TOKEN_ID, TEMPO_USDC_TOKEN_ID},
        known_assets::TEMPO_BRIDGED_USDC,
        swap::SwapData,
    };
    use serde_json::Value;

    use super::*;
    use crate::contracts::{ITIP20, ITempoFeeManager};
    use crate::fee::FEE_MANAGER_ADDRESS;
    use crate::testkit::mock_tempo_generic_input;

    fn encode_currency(currency: &str) -> serde_json::Value {
        serde_json::json!(encode_prefixed(ITIP20::currencyCall::abi_encode_returns(&currency.to_string())))
    }

    fn user_token_response(token: Address) -> serde_json::Value {
        serde_json::json!(encode_prefixed(ITempoFeeManager::userTokensCall::abi_encode_returns(&token)))
    }

    fn new_calculator<F>(handler: F) -> TempoFeeCalculator<MockClient>
    where
        F: Fn(&str, &Value) -> Result<Value, ClientError> + Send + Sync + 'static,
    {
        TempoFeeCalculator::new(EthereumClient::new(mock_jsonrpc_client(handler), EVMChain::Tempo))
    }

    fn swap_input(from_asset: Asset) -> TransactionLoadInput {
        TransactionLoadInput::mock_evm(
            TransactionInputType::Swap(
                from_asset,
                TEMPO_BRIDGED_USDC.clone(),
                SwapData::mock_with_provider_data(SwapProvider::UniswapV4, "abcd", None),
            ),
            "0",
        )
    }

    #[tokio::test]
    async fn test_calculate_fee() -> Result<(), Box<dyn Error + Sync + Send>> {
        let calculator = new_calculator(|_, params| {
            if params[0]["to"].as_str().unwrap().eq_ignore_ascii_case(FEE_MANAGER_ADDRESS) {
                Ok(user_token_response(Address::ZERO))
            } else {
                Ok(encode_currency(USD_CURRENCY))
            }
        });
        let gas_limit = BigInt::from(21_000u64);
        let mut input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(TEMPO_BRIDGED_USDC.clone()), "1000000");
        input.gas_price = GasPriceType::eip1559(BigInt::from(20_000_000_001u64), BigInt::from(0u64));
        let fee = calculator.calculate_fee(&input, &gas_limit).await?;

        assert_eq!(fee.fee, BigInt::from(421u64));
        assert_eq!(fee.fee_asset, TEMPO_BRIDGED_USDC.id);
        assert_eq!(fee.gas_limit, gas_limit);
        assert_eq!(fee.gas_price_type.gas_price(), BigInt::from(20_000_000_001u64));

        let token_asset = TEMPO_BRIDGED_USDC.clone();
        let input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(token_asset.clone()), "1000000");
        let fee = calculator.calculate_fee(&input, &BigInt::from(TOKEN_TRANSFER_GAS_LIMIT)).await?;
        assert_eq!(fee.fee_asset, token_asset.id);

        let input = TransactionLoadInput::mock_evm(mock_tempo_generic_input("0x0000000000000000000000000000000000000001", vec![0xab, 0xcd]), "0");
        let fee = calculator.calculate_fee(&input, &BigInt::from(100_000u64)).await?;
        assert_eq!(fee.fee_asset, TEMPO_PATHUSD_ASSET_ID.clone());

        Ok(())
    }

    #[tokio::test]
    async fn test_set_user_token_overrides_account_preference() {
        let calculator = new_calculator(|method, _| {
            assert_eq!(method, "eth_call");
            Ok(encode_currency(USD_CURRENCY))
        });
        let calldata = ITempoFeeManager::setUserTokenCall {
            token: TEMPO_PATHUSD_TOKEN_ID.parse().unwrap(),
        }
        .abi_encode();
        let input = TransactionLoadInput::mock_evm(mock_tempo_generic_input(FEE_MANAGER_ADDRESS, calldata), "0");

        let fee_asset = calculator.fee_asset(&input).await.unwrap();

        assert_eq!(fee_asset, TEMPO_PATHUSD_ASSET_ID.clone());

        let invalid_calculator = new_calculator(|_, _| Ok(encode_currency("BTC")));
        assert!(invalid_calculator.fee_asset(&input).await.is_err());
    }

    #[tokio::test]
    async fn test_account_fee_token_requires_usd_currency() {
        let account_token = TEMPO_USDC_TOKEN_ID.parse().unwrap();
        let calculator = new_calculator(move |_, params| {
            if params[0]["to"].as_str().unwrap().eq_ignore_ascii_case(FEE_MANAGER_ADDRESS) {
                Ok(user_token_response(account_token))
            } else {
                Ok(encode_currency("BTC"))
            }
        });
        let input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(TEMPO_BRIDGED_USDC.clone()), "1000000");

        assert!(calculator.fee_asset(&input).await.is_err());
    }

    #[tokio::test]
    async fn test_swap_fee_asset_requires_usd_currency() {
        let usd_calculator = new_calculator(|_, _| Ok(encode_currency(USD_CURRENCY)));
        let usdc = TEMPO_BRIDGED_USDC.clone();
        assert_eq!(usd_calculator.fee_asset(&swap_input(usdc.clone())).await.unwrap(), usdc.id);

        let btc_calculator = new_calculator(|_, _| Ok(encode_currency("BTC")));
        let cbbtc = Asset::mock_with_params(
            Chain::Tempo,
            Some("0x20C000000000000000000000c412Ec89D0c08be5".to_string()),
            "Coinbase Wrapped BTC".to_string(),
            "cbBTC".to_string(),
            6,
            AssetType::TIP20,
        );
        assert_eq!(btc_calculator.fee_asset(&swap_input(cbbtc)).await.unwrap(), TEMPO_PATHUSD_ASSET_ID.clone());
    }
}
