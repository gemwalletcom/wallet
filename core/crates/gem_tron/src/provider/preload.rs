use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;

use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetSubtype, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, TransferDataOutputAction, TronStakeData, decode_hex,
    swap::{SwapData, SwapQuoteData, SwapQuoteDataType},
};

use crate::{
    address::TronAddress,
    models::{ChainParameter, TriggerSmartContractData, account::TronAccountUsage},
    provider::preload_mapper::{
        FeeEstimateContext, calculate_stake_fee_rate, calculate_transfer_token_fee_rate, calculate_transfer_token_fee_rate_with_data, map_stake_data, native_transfer_fee,
    },
    rpc::client::TronClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (block, chain_parameters, account_usage, is_new_account, stake_data) = futures::try_join!(
            self.get_tron_block(),
            self.get_chain_parameters(),
            self.get_account_usage(&input.sender_address),
            self.get_is_new_account_for_input_type(&input),
            self.get_stake_data(&input)
        )?;

        let block = block.block_header.raw_data;
        let metadata = TransactionLoadMetadata::Tron {
            block_number: block.number,
            block_version: block.version,
            block_timestamp: block.timestamp,
            transaction_tree_root: block.tx_trie_root.clone(),
            parent_hash: block.parent_hash.clone(),
            witness_address: block.witness_address.clone(),
            stake_data,
        };

        let fee_context = FeeEstimateContext {
            chain_parameters: &chain_parameters,
            account_usage: &account_usage,
            is_new_account,
        };
        let has_memo = input.get_memo().is_some();
        let fee = match &input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::TransferNft(asset, _) | TransactionInputType::Account(asset, _) => match &asset.id.token_id {
                None => native_transfer_fee(&fee_context, has_memo)?,
                Some(token_id) => {
                    self.estimate_token_transfer_fee(
                        input.sender_address.clone(),
                        input.destination_address.clone(),
                        token_id.clone(),
                        input.value.clone(),
                        &chain_parameters,
                        &account_usage,
                        input.get_memo().map(|memo| memo.len() as u64),
                    )
                    .await?
                }
            },
            TransactionInputType::Generic(_, _, extra) => match extra.output_action {
                TransferDataOutputAction::Send => match self
                    .estimate_fee_with_data(&input.sender_address, extra.data.as_deref(), &chain_parameters, &account_usage)
                    .await?
                {
                    Some(fee) => fee,
                    None => native_transfer_fee(&fee_context, has_memo)?,
                },
                TransferDataOutputAction::Sign => native_transfer_fee(&fee_context, false)?,
            },
            TransactionInputType::Stake(_asset, stake_type) => TransactionFee::new_from_fee(calculate_stake_fee_rate(&chain_parameters, &account_usage, stake_type)?),
            TransactionInputType::Swap(from_asset, _, swap_data) => self.estimate_swap_fee(&input, from_asset, swap_data, &fee_context, input.get_memo()).await?,
            _ => native_transfer_fee(&fee_context, has_memo)?,
        };

        Ok(TransactionLoadData { fee, metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

fn has_swap_quote_memo(input_memo: Option<&str>, data: &SwapQuoteData) -> bool {
    input_memo.is_some() || data.memo.as_deref().is_some_and(|memo| !memo.is_empty())
}

fn swap_contract_memo_data_bytes(input_memo: Option<&str>, data: &SwapQuoteData) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
    if let Some(memo) = data.memo.as_deref().filter(|memo| !memo.is_empty()) {
        let bytes = decode_hex(memo).map_err(|_| "invalid Tron swap memo")?;
        return Ok(Some(bytes.len() as u64));
    }

    Ok(input_memo.map(|memo| memo.len() as u64))
}

impl<C: Client> TronClient<C> {
    async fn estimate_swap_fee(
        &self,
        input: &TransactionLoadInput,
        from_asset: &Asset,
        swap_data: &SwapData,
        fee_context: &FeeEstimateContext<'_>,
        input_memo: Option<&str>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        match &swap_data.data.data_type {
            SwapQuoteDataType::Contract => self.estimate_contract_swap_fee(&input.sender_address, from_asset, swap_data, fee_context, input_memo).await,
            SwapQuoteDataType::Transfer => self.estimate_transfer_swap_fee(input, from_asset, swap_data, fee_context, input_memo).await,
        }
    }

    async fn estimate_contract_swap_fee(
        &self,
        sender_address: &str,
        from_asset: &Asset,
        swap_data: &SwapData,
        fee_context: &FeeEstimateContext<'_>,
        input_memo: Option<&str>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        if !swap_data.data.data.is_empty() {
            let memo_data_bytes = swap_contract_memo_data_bytes(input_memo, &swap_data.data)?;
            return self
                .estimate_contract_call_fee(sender_address, &swap_data.data, fee_context.chain_parameters, fee_context.account_usage, memo_data_bytes)
                .await;
        }
        if from_asset.id.token_id.is_some() {
            return Err("Tron token contract swap calldata is required".into());
        }

        native_transfer_fee(fee_context, has_swap_quote_memo(input_memo, &swap_data.data))
    }

    async fn estimate_transfer_swap_fee(
        &self,
        input: &TransactionLoadInput,
        from_asset: &Asset,
        swap_data: &SwapData,
        fee_context: &FeeEstimateContext<'_>,
        input_memo: Option<&str>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        match &from_asset.id.token_id {
            None => native_transfer_fee(fee_context, has_swap_quote_memo(input_memo, &swap_data.data)),
            Some(token_id) => {
                self.estimate_token_transfer_fee(
                    input.sender_address.clone(),
                    swap_data.data.to.clone(),
                    token_id.clone(),
                    input.value.clone(),
                    fee_context.chain_parameters,
                    fee_context.account_usage,
                    input_memo.map(|memo| memo.len() as u64),
                )
                .await
            }
        }
    }

    async fn estimate_token_transfer_fee(
        &self,
        sender_address: String,
        destination_address: String,
        token_id: String,
        value: String,
        chain_parameters: &[ChainParameter],
        account_usage: &TronAccountUsage,
        memo_data_bytes: Option<u64>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let destination_parameter = TronAddress::parse(&destination_address)?.abi_address_parameter();
        let estimated_energy = self.estimate_trc20_transfer_gas(sender_address, token_id, destination_parameter, value).await?;
        let token_fee = calculate_transfer_token_fee_rate_with_data(
            chain_parameters,
            account_usage,
            estimated_energy,
            memo_data_bytes.unwrap_or_default(),
            memo_data_bytes.is_some(),
        )?;

        Ok(TransactionFee::new_gas_price_type(
            GasPriceType::regular(BigInt::from(token_fee.energy_price)),
            BigInt::from(token_fee.fee),
            BigInt::from(token_fee.fee_limit),
            HashMap::new(),
        ))
    }

    async fn estimate_contract_call_fee(
        &self,
        sender_address: &str,
        data: &SwapQuoteData,
        chain_parameters: &[ChainParameter],
        account_usage: &TronAccountUsage,
        memo_data_bytes: Option<u64>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let contract_data = TriggerSmartContractData {
            contract_address: data.to.clone(),
            data: data.data.clone(),
            owner_address: sender_address.to_string(),
            fee_limit: None,
            call_value: Some(data.value.parse::<u64>()?).filter(|value| *value > 0),
        };
        let estimated_energy = self.estimate_energy_with_data(&contract_data).await?;
        let token_fee = calculate_transfer_token_fee_rate_with_data(
            chain_parameters,
            account_usage,
            estimated_energy,
            memo_data_bytes.unwrap_or_default(),
            memo_data_bytes.is_some(),
        )?;

        Ok(TransactionFee::new_gas_price_type(
            GasPriceType::regular(BigInt::from(token_fee.energy_price)),
            BigInt::from(token_fee.fee),
            BigInt::from(token_fee.fee_limit),
            HashMap::new(),
        ))
    }

    async fn estimate_fee_with_data(
        &self,
        sender_address: &str,
        data: Option<&[u8]>,
        chain_parameters: &[ChainParameter],
        account_usage: &TronAccountUsage,
    ) -> Result<Option<TransactionFee>, Box<dyn Error + Send + Sync>> {
        let Some(parsed) = TriggerSmartContractData::from_payload(data, sender_address)? else {
            return Ok(None);
        };

        let estimated_energy = self.estimate_energy_with_data(&parsed).await?;
        let token_fee = calculate_transfer_token_fee_rate(chain_parameters, account_usage, estimated_energy)?;

        Ok(Some(TransactionFee::new_gas_price_type(
            GasPriceType::regular(BigInt::from(token_fee.energy_price)),
            BigInt::from(token_fee.fee),
            BigInt::from(token_fee.fee_limit),
            HashMap::new(),
        )))
    }

    async fn get_is_new_account_for_input_type(&self, input: &TransactionLoadInput) -> Result<bool, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _)
            | TransactionInputType::Swap(asset, _, _) => match asset.id.token_subtype() {
                AssetSubtype::NATIVE => self.is_new_account(input.input_type.swap_to_address().unwrap_or(&input.destination_address)).await,
                AssetSubtype::TOKEN => Ok(false),
            },
            _ => Ok(false),
        }
    }

    async fn get_stake_data(&self, input: &TransactionLoadInput) -> Result<TronStakeData, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Stake(asset, stake_type) => {
                let account = self.get_account(&input.sender_address).await?;
                let raw_amount = BigNumberFormatter::value_as_u64(&input.value, 0)?;
                let vote_amount = BigNumberFormatter::value_as_u64(&input.value, asset.decimals as u32)?;
                map_stake_data(&account, stake_type, raw_amount, vote_amount)
            }
            _ => Ok(TronStakeData::Votes(vec![])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{has_swap_quote_memo, swap_contract_memo_data_bytes};
    use primitives::swap::SwapQuoteData;

    #[test]
    fn test_swap_memo_fee_preload_uses_quote_memo_bytes() {
        let mut data = SwapQuoteData::new_contract("TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi".to_string(), "0".to_string(), String::new(), None, None);

        assert!(!has_swap_quote_memo(None, &data));
        assert_eq!(swap_contract_memo_data_bytes(None, &data).unwrap(), None);

        assert!(has_swap_quote_memo(Some("memo"), &data));
        assert_eq!(swap_contract_memo_data_bytes(Some("memo"), &data).unwrap(), Some(4));

        data.memo = Some(String::new());
        assert!(!has_swap_quote_memo(None, &data));
        assert_eq!(swap_contract_memo_data_bytes(None, &data).unwrap(), None);

        data.memo = Some("0x010203".to_string());
        assert!(has_swap_quote_memo(None, &data));
        assert!(has_swap_quote_memo(Some("memo"), &data));
        assert_eq!(swap_contract_memo_data_bytes(Some("memo"), &data).unwrap(), Some(3));

        data.memo = Some("0xzz".to_string());
        assert_eq!(swap_contract_memo_data_bytes(None, &data).unwrap_err().to_string(), "invalid Tron swap memo");
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_USDT_TOKEN_ID, create_test_client};
    use chain_traits::ChainTransactionLoad;
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, AssetType, Chain};

    #[tokio::test]
    async fn test_get_transaction_load_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let asset = Asset::from_chain(Chain::Tron);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(asset));
        let input = TransactionLoadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "TGas3vJWx6R9wZEq66T3p7T5QAkXHRzh2q".to_string(),
            ..input
        };

        let result = client.get_transaction_load(input).await?;

        assert!(result.fee.fee > BigInt::from(0), "Transfer fee should be calculated");

        if let TransactionLoadMetadata::Tron { block_number, .. } = result.metadata {
            assert!(block_number > 0, "Block number should be positive");
        } else {
            panic!("Expected Tron metadata");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_load_token_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let asset_id = AssetId::from(Chain::Tron, Some(TEST_USDT_TOKEN_ID.to_string()));
        let asset = Asset::new(asset_id, "Tether USD".to_string(), "USDT".to_string(), 6, AssetType::TRC20);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(asset));
        let input = TransactionLoadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "TGas3vJWx6R9wZEq66T3p7T5QAkXHRzh2q".to_string(),
            ..input
        };

        let result = client.get_transaction_load(input).await?;

        assert!(result.fee.gas_limit > result.fee.fee, "Fee limit should be greater than estimated fee");
        assert!(result.fee.gas_limit > BigInt::from(0), "Gas limit should be greater than 0");

        if let TransactionLoadMetadata::Tron { block_number, .. } = result.metadata {
            assert!(block_number > 0, "Block number should be positive");
        } else {
            panic!("Expected Tron metadata");
        }

        Ok(())
    }
}
