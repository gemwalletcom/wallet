use std::error::Error;

use alloy_primitives::Address;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::provider::preload;
use gem_evm::provider::preload_mapper::TransactionParams;
use gem_evm::rpc::{EthereumClient, EthereumProvider, EvmFeeCalculator};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, TransactionFee, TransactionInputType, TransactionLoadInput};

use crate::contracts::{ITIP20, ITempoFeeManager};
use crate::fee::{FEE_MANAGER_ADDRESS, USD_CURRENCY, decode_set_user_fee_token, scale_fee_to_token_units};
use crate::mapper::map_asset_id;

pub struct TempoFeeCalculator<C: Client + Clone> {
    provider: EthereumProvider<C>,
}

#[async_trait]
impl<C: Client + Clone> EvmFeeCalculator for TempoFeeCalculator<C> {
    async fn calculate_fee(&self, input: &TransactionLoadInput, _params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
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
    pub fn new(client: EthereumClient<C>) -> Self {
        Self {
            provider: EthereumProvider::new_rpc_only(client),
        }
    }

    async fn fee_asset(&self, input: &TransactionLoadInput) -> Result<AssetId, Box<dyn Error + Sync + Send>> {
        let input_type = &input.input_type;
        if let Some(token) = decode_set_user_fee_token(input_type) {
            let token_id = token.to_checksum(None);
            if self.tip20_currency(&token_id).await? != USD_CURRENCY {
                return Err("Tempo fee token must use USD currency".into());
            }
            return Ok(map_asset_id(AssetId::from_token(Chain::Tempo, &token_id)));
        }

        if !matches!(input_type, TransactionInputType::Swap(_, _, _)) {
            let account_fee_token = self.user_fee_token(&input.sender_address).await?;
            if !account_fee_token.is_zero() {
                return Ok(map_asset_id(AssetId::from_token(Chain::Tempo, &account_fee_token.to_checksum(None))));
            }
        }

        let fee_asset = input_type.get_fee_asset_id();
        let Some(token_id) = fee_asset.token_id.as_deref() else {
            return Ok(fee_asset);
        };
        if self.tip20_currency(token_id).await? != USD_CURRENCY {
            return Ok(AssetId::from_chain(Chain::Tempo));
        }

        Ok(map_asset_id(fee_asset))
    }
    async fn user_fee_token(&self, address: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        self.provider
            .call_contract(FEE_MANAGER_ADDRESS.parse()?, ITempoFeeManager::userTokensCall { user: address.parse()? })
            .await
    }

    async fn tip20_currency(&self, token_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.provider.call_contract(token_id.parse()?, ITIP20::currencyCall {}).await
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, hex::encode_prefixed};
    use alloy_sol_types::SolCall;
    use gem_client::ClientError;
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{Asset, Chain, EVMChain, GasPriceType, SwapProvider, TransactionLoadInput, asset_constants::TEMPO_PATHUSD_TOKEN_ID, swap::SwapData};
    use serde_json::Value;

    use super::*;
    use crate::contracts::{ITIP20, ITempoFeeManager};
    use crate::fee::FEE_MANAGER_ADDRESS;
    use crate::testkit::{mock_tempo_cbbtc_asset, mock_tempo_generic_input};

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
                Asset::from_chain(Chain::Tempo),
                SwapData::mock_with_provider_data(SwapProvider::UniswapV4, "abcd", None),
            ),
            "0",
        )
    }

    async fn calculate_fee(calculator: &TempoFeeCalculator<MockClient>, input: &TransactionLoadInput, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        EvmFeeCalculator::calculate_fee(calculator, input, &TransactionParams::new(String::new(), vec![], BigInt::ZERO), gas_limit).await
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
        let mut input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo)), "1000000");
        input.gas_price = GasPriceType::eip1559(BigInt::from(20_000_000_001u64), BigInt::from(0u64));
        let fee = calculate_fee(&calculator, &input, &gas_limit).await?;

        assert_eq!(fee.fee, BigInt::from(421u64));
        assert_eq!(fee.fee_asset, AssetId::from_chain(Chain::Tempo));
        assert_eq!(fee.gas_limit, gas_limit);
        assert_eq!(fee.gas_price_type.gas_price(), BigInt::from(20_000_000_001u64));

        let token_asset = Asset::mock_tempo_usdc();
        let input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(token_asset.clone()), "1000000");
        let fee = calculate_fee(&calculator, &input, &BigInt::from(65_000u64)).await?;
        assert_eq!(fee.fee_asset, token_asset.id);

        let input = TransactionLoadInput::mock_evm(mock_tempo_generic_input("0x0000000000000000000000000000000000000001", vec![0xab, 0xcd]), "0");
        let fee = calculate_fee(&calculator, &input, &BigInt::from(100_000u64)).await?;
        assert_eq!(fee.fee_asset, AssetId::from_chain(Chain::Tempo));

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

        assert_eq!(fee_asset, AssetId::from_chain(Chain::Tempo));

        let invalid_calculator = new_calculator(|_, _| Ok(encode_currency("BTC")));
        assert!(invalid_calculator.fee_asset(&input).await.is_err());
    }

    #[tokio::test]
    async fn test_swap_fee_asset_requires_usd_currency() {
        let usd_calculator = new_calculator(|_, _| Ok(encode_currency(USD_CURRENCY)));
        let usdc = Asset::mock_tempo_usdc();
        assert_eq!(usd_calculator.fee_asset(&swap_input(usdc.clone())).await.unwrap(), usdc.id);

        let btc_calculator = new_calculator(|_, _| Ok(encode_currency("BTC")));
        assert_eq!(
            btc_calculator.fee_asset(&swap_input(mock_tempo_cbbtc_asset())).await.unwrap(),
            AssetId::from_chain(Chain::Tempo)
        );
    }
}
