use std::collections::HashMap;
use std::error::Error;
use std::str;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures_util::future::{try_join, try_join5};
use num_bigint::BigInt;

use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::{
    Asset, AssetId, AssetSubtype, Chain, FeePriority, FeeRate, GasPriceType, OptionStringExt, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
    TransferDataOutputAction, TronStakeData, decode_hex,
    swap::{SwapData, SwapQuoteData, SwapQuoteDataType},
};

use crate::{
    address::TronAddress,
    models::{ChainParameter, TriggerSmartContractData, TronContractInfo, account::TronAccountUsage},
    provider::preload_mapper::{FEE_LIMIT_BUFFER_PERCENT, FeeEstimateContext, SMART_CONTRACT_FEE_LIMIT_BUFFER_PERCENT, calculate_stake_fee_rate, calculate_token_fee_rate_with_data, map_stake_data, native_transfer_fee},
    rpc::{TronClient, TronProvider},
    transaction::{TronContract, TronTriggerSmartContract, decode_wallet_connect_trigger_smart_contract},
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronProvider<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (block, chain_parameters, account_usage, is_new_account, stake_data) = try_join5(
            self.get_tron_block(),
            self.get_chain_parameters(),
            self.get_account_usage(&input.sender_address),
            self.get_is_new_account_for_input_type(&input),
            self.get_stake_data(&input),
        )
        .await?;

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
        let memo_data_bytes = memo_data_bytes(&input)?;
        let fee = match &input.input_type {
            TransactionInputType::Transfer { asset } | TransactionInputType::Withdrawal { asset } | TransactionInputType::TransferNft { asset, .. } | TransactionInputType::Account { asset, .. } => match &asset.id.token_id {
                None => native_transfer_fee(&fee_context, memo_data_bytes)?,
                Some(token_id) => {
                    let call = TronTriggerSmartContract::token_transfer(TronAddress::parse(&input.sender_address)?, token_id, &input.destination_address, &input.value)?;
                    self.estimate_token_transfer_fee(&call, &fee_context, memo_data_bytes).await?
                }
            },
            TransactionInputType::Generic { extra, .. } | TransactionInputType::Payment { extra, .. } => match extra.output_action {
                TransferDataOutputAction::Send => match self.estimate_fee_with_data(extra.data.as_deref(), &fee_context).await? {
                    Some(fee) => fee,
                    None => native_transfer_fee(&fee_context, memo_data_bytes)?,
                },
                TransferDataOutputAction::Sign => native_transfer_fee(&fee_context, None)?,
            },
            TransactionInputType::Stake { .. } => TransactionFee::new_from_fee(calculate_stake_fee_rate(&chain_parameters, &account_usage)?, AssetId::from_chain(Chain::Tron)),
            TransactionInputType::TokenApprove { approval_data: approval, .. } => {
                let call = TronTriggerSmartContract::token_approval(TronAddress::parse(&input.sender_address)?, approval)?;
                self.estimate_smart_contract_fee(call.trigger_smart_contract_data(None), &fee_context, None).await?
            }
            TransactionInputType::Swap { from_asset, swap_data, .. } => self.estimate_swap_fee(&input, from_asset, swap_data, &fee_context, memo_data_bytes).await?,
            _ => native_transfer_fee(&fee_context, memo_data_bytes)?,
        };

        Ok(TransactionLoadData { fee, metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

fn memo_data_bytes(input: &TransactionLoadInput) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
    let swap_memo = input
        .input_type
        .get_swap_data()
        .ok()
        .and_then(|swap_data| swap_data.data.memo.as_deref().non_empty().map(|memo| (&swap_data.data.data_type, memo)));
    let bytes = match swap_memo {
        Some((SwapQuoteDataType::Contract, memo)) => Some(decode_hex(memo).map_err(|_| "invalid Tron swap memo")?.len()),
        Some((SwapQuoteDataType::Transfer, memo)) => Some(memo.len()),
        None => input.get_memo().map(str::len),
    };
    Ok(bytes.map(|bytes| bytes as u64))
}

fn needs_approval_energy_estimate(data: &SwapQuoteData) -> Option<u64> {
    if data.approval.is_none() && data.permit2.is_none() {
        return None;
    }
    data.gas_limit.as_deref()?.parse::<u64>().ok().filter(|energy| *energy > 0)
}

impl<C: Client> TronClient<C> {
    async fn estimate_swap_fee(&self, input: &TransactionLoadInput, from_asset: &Asset, swap_data: &SwapData, fee_context: &FeeEstimateContext<'_>, memo_data_bytes: Option<u64>) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        match &swap_data.data.data_type {
            SwapQuoteDataType::Contract => self.estimate_contract_swap_fee(&input.sender_address, from_asset, swap_data, fee_context, memo_data_bytes).await,
            SwapQuoteDataType::Transfer => self.estimate_transfer_swap_fee(input, from_asset, swap_data, fee_context, memo_data_bytes).await,
        }
    }

    async fn estimate_contract_swap_fee(&self, sender_address: &str, from_asset: &Asset, swap_data: &SwapData, fee_context: &FeeEstimateContext<'_>, memo_data_bytes: Option<u64>) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let owner = TronAddress::parse(sender_address)?;
        let swap_contract = TronContract::contract_swap(owner, from_asset, &swap_data.data)?;
        let swap_fee = self.estimate_contract_swap_call_fee(&swap_contract, swap_data, fee_context, memo_data_bytes);
        if let Some(approval) = &swap_data.data.approval {
            let approval_call = TronTriggerSmartContract::token_approval(owner, approval)?;
            let approval_fee = self.estimate_smart_contract_fee(approval_call.trigger_smart_contract_data(None), fee_context, None);
            let (swap_fee, approval_fee) = try_join(swap_fee, approval_fee).await?;
            let fee = &approval_fee.fee + &swap_fee.fee;
            let gas_limit = &approval_fee.gas_limit + &swap_fee.gas_limit;
            return Ok(TransactionFee {
                fee,
                gas_limit,
                gas_price_type: approval_fee.gas_price_type,
                options: approval_fee.options,
                fee_asset: AssetId::from_chain(Chain::Tron),
            });
        }
        swap_fee.await
    }

    async fn estimate_contract_swap_call_fee(&self, swap_contract: &TronContract, swap_data: &SwapData, fee_context: &FeeEstimateContext<'_>, memo_data_bytes: Option<u64>) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let TronContract::TriggerSmart(call) = swap_contract else {
            return native_transfer_fee(fee_context, memo_data_bytes);
        };
        if let Some(estimated_energy) = needs_approval_energy_estimate(&swap_data.data) {
            let caller_energy = self.caller_energy(&self.get_contract(&call.contract.to_string()).await?, estimated_energy).await?;
            return contract_fee_from_energy(fee_context.chain_parameters, fee_context.account_usage, (caller_energy, estimated_energy), call.data.len() as u64, memo_data_bytes);
        }
        self.estimate_smart_contract_fee(call.trigger_smart_contract_data(None), fee_context, memo_data_bytes).await
    }

    async fn estimate_transfer_swap_fee(
        &self,
        input: &TransactionLoadInput,
        from_asset: &Asset,
        swap_data: &SwapData,
        fee_context: &FeeEstimateContext<'_>,
        memo_data_bytes: Option<u64>,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        match &from_asset.id.token_id {
            None => native_transfer_fee(fee_context, memo_data_bytes),
            Some(token_id) => {
                let call = TronTriggerSmartContract::token_transfer(TronAddress::parse(&input.sender_address)?, token_id, &swap_data.data.to, &input.value)?;
                self.estimate_token_transfer_fee(&call, fee_context, memo_data_bytes).await
            }
        }
    }

    async fn estimate_token_transfer_fee(&self, call: &TronTriggerSmartContract, fee_context: &FeeEstimateContext<'_>, memo_data_bytes: Option<u64>) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let estimated_energy = self.estimate_energy_with_data(&call.trigger_smart_contract_data(None)).await?;
        transaction_fee_from_energy_estimate(fee_context.chain_parameters, fee_context.account_usage, estimated_energy, call.data.len() as u64, memo_data_bytes, FEE_LIMIT_BUFFER_PERCENT)
    }

    async fn estimate_smart_contract_fee(&self, contract_data: TriggerSmartContractData, fee_context: &FeeEstimateContext<'_>, memo_data_bytes: Option<u64>) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let (estimated_energy, contract) = try_join(self.estimate_energy_with_data(&contract_data), self.get_contract(&contract_data.contract_address)).await?;
        let caller_energy = self.caller_energy(&contract, estimated_energy).await?;
        contract_fee_from_energy(fee_context.chain_parameters, fee_context.account_usage, (caller_energy, estimated_energy), contract_data.data.len() as u64 / 2, memo_data_bytes)
    }

    async fn caller_energy(&self, contract: &TronContractInfo, energy: u64) -> Result<u64, Box<dyn Error + Send + Sync>> {
        if contract.caller_pays_all_energy() {
            return Ok(energy);
        }
        Ok(contract.caller_energy(energy, &self.get_account_usage(&contract.origin_address).await?))
    }

    async fn estimate_fee_with_data(&self, data: Option<&[u8]>, fee_context: &FeeEstimateContext<'_>) -> Result<Option<TransactionFee>, Box<dyn Error + Send + Sync>> {
        let Some(data) = data else {
            return Ok(None);
        };
        let Some(contract_data) = decode_wallet_connect_trigger_smart_contract(str::from_utf8(data)?)? else {
            return Ok(None);
        };

        Ok(Some(self.estimate_smart_contract_fee(contract_data, fee_context, None).await?))
    }

    async fn get_is_new_account_for_input_type(&self, input: &TransactionLoadInput) -> Result<bool, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Transfer { asset }
            | TransactionInputType::Withdrawal { asset }
            | TransactionInputType::TransferNft { asset, .. }
            | TransactionInputType::Account { asset, .. }
            | TransactionInputType::Swap { from_asset: asset, .. } => match asset.id.token_subtype() {
                AssetSubtype::NATIVE => self.is_new_account(input.input_type.swap_to_address().unwrap_or(&input.destination_address)).await,
                AssetSubtype::TOKEN => Ok(false),
            },
            _ => Ok(false),
        }
    }

    async fn get_stake_data(&self, input: &TransactionLoadInput) -> Result<TronStakeData, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Stake { asset, stake_type } => {
                let account = self.get_account(&input.sender_address).await?;
                let raw_amount = BigNumberFormatter::value_as_u64(&input.value, 0)?;
                let vote_amount = BigNumberFormatter::value_as_u64(&input.value, asset.decimals)?;
                map_stake_data(&account, stake_type, raw_amount, vote_amount)
            }
            _ => Ok(TronStakeData::Votes { votes: vec![] }),
        }
    }
}

fn transaction_fee_from_energy_estimate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    estimated_energy: u64,
    call_data_bytes: u64,
    memo_data_bytes: Option<u64>,
    buffer_percent: u64,
) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
    let data_bytes = call_data_bytes + memo_data_bytes.unwrap_or(0);
    let token_fee = calculate_token_fee_rate_with_data(chain_parameters, account_usage, estimated_energy, data_bytes, memo_data_bytes.is_some(), buffer_percent)?;
    Ok(TransactionFee::new_gas_price_type(
        GasPriceType::regular(BigInt::from(token_fee.energy_price)),
        BigInt::from(token_fee.fee),
        BigInt::from(token_fee.fee_limit),
        HashMap::new(),
        AssetId::from_chain(Chain::Tron),
    ))
}

fn contract_fee_from_energy(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    (caller_energy, total_energy): (u64, u64),
    call_data_bytes: u64,
    memo_data_bytes: Option<u64>,
) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
    let caller_fee = transaction_fee_from_energy_estimate(chain_parameters, account_usage, caller_energy, call_data_bytes, memo_data_bytes, SMART_CONTRACT_FEE_LIMIT_BUFFER_PERCENT)?;
    let total_fee = transaction_fee_from_energy_estimate(chain_parameters, account_usage, total_energy, call_data_bytes, memo_data_bytes, SMART_CONTRACT_FEE_LIMIT_BUFFER_PERCENT)?;
    Ok(TransactionFee {
        gas_limit: total_fee.gas_limit,
        ..caller_fee
    })
}

#[cfg(test)]
mod tests {
    use super::memo_data_bytes;
    use crate::models::TriggerSmartContractRequest;
    use crate::rpc::TronProvider;
    use chain_traits::ChainTransactionLoad;
    use num_bigint::{BigInt, BigUint};
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;
    use primitives::swap::{ApprovalData, SwapQuoteData};
    use primitives::testkit::signer_mock::TEST_TRON_SENDER;
    use primitives::{Asset, Chain, TransactionInputType, TransactionLoadInput};

    const ROUTER: &str = "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi";
    const SWAP_CALLDATA: &str = "7b9392320000000000000000000000005cd0fb0ab3ce40f3051414c604b27756e69e43db";
    const SWAP_MEMO: &str = "=:ETH.ETH:0x1111111111111111111111111111111111111111:0/1/0:g1:50";

    #[tokio::test]
    async fn test_get_transaction_load_estimates_the_signed_calls() {
        let (provider, requests) = TronProvider::mock_estimates();
        let approval = ApprovalData::make(TRON_USDT_TOKEN_ID, ROUTER, BigUint::ZERO, true);
        let token_swap = SwapQuoteData {
            approval: Some(approval),
            ..SwapQuoteData::mock_contract_call(ROUTER, "0", SWAP_CALLDATA, None)
        };
        let native_swap = SwapQuoteData::mock_contract_call(ROUTER, "1000", SWAP_CALLDATA, None);
        let native_memo_swap = SwapQuoteData::new_transfer(ROUTER.to_string(), BigUint::from(1000u32), Some(SWAP_MEMO.to_string()));

        let mut fees = Vec::new();
        for input_type in [
            TransactionInputType::Transfer { asset: Asset::mock_tron_usdt() },
            TransactionInputType::mock_swap(Asset::mock_tron_usdt(), token_swap),
            TransactionInputType::mock_swap(Asset::from_chain(Chain::Tron), native_swap),
            TransactionInputType::mock_swap(Asset::from_chain(Chain::Tron), native_memo_swap),
        ] {
            fees.push(provider.get_transaction_load(TransactionLoadInput::mock_tron(input_type, 1000)).await.unwrap().fee.fee);
        }

        assert_eq!(fees, [BigInt::from(8_082_200), BigInt::from(17_418_000), BigInt::from(8_693_000), BigInt::from(1_364_000)]);
        assert_eq!(
            *requests.lock().unwrap(),
            [
                TriggerSmartContractRequest::mock(
                    TEST_TRON_SENDER,
                    TRON_USDT_TOKEN_ID,
                    "a9059cbb0000000000000000000000413e1451cdb84d440345de6195b0384d1b77aa4eaa00000000000000000000000000000000000000000000000000000000000003e8",
                    None
                ),
                TriggerSmartContractRequest::mock(TEST_TRON_SENDER, ROUTER, SWAP_CALLDATA, None),
                TriggerSmartContractRequest::mock(
                    TEST_TRON_SENDER,
                    TRON_USDT_TOKEN_ID,
                    "095ea7b30000000000000000000000412523ae929fecd9d665f472f59b99a8ce6b179510ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    None
                ),
                TriggerSmartContractRequest::mock(TEST_TRON_SENDER, ROUTER, SWAP_CALLDATA, Some(1000)),
            ]
        );
    }

    #[test]
    fn test_memo_data_bytes() {
        let tron = Asset::from_chain(Chain::Tron);
        let transfer = TransactionLoadInput::mock_tron(TransactionInputType::Transfer { asset: tron.clone() }, 1000);
        let with_memo = TransactionLoadInput {
            memo: Some("memo".to_string()),
            ..transfer.clone()
        };

        assert_eq!(memo_data_bytes(&transfer).unwrap(), None);
        assert_eq!(memo_data_bytes(&with_memo).unwrap(), Some(4));

        let transfer_swap = SwapQuoteData::new_transfer(ROUTER.to_string(), BigUint::from(1000u32), Some(SWAP_MEMO.to_string()));
        assert_eq!(
            memo_data_bytes(&TransactionLoadInput::mock_tron(TransactionInputType::mock_swap(tron.clone(), transfer_swap), 1000)).unwrap(),
            Some(64),
            "a transfer swap signs its memo as text"
        );

        for (swap_memo, bytes) in [(Some("0x010203"), Some(3)), (Some(""), Some(4)), (None, Some(4))] {
            let input_type = TransactionInputType::mock_swap(tron.clone(), SwapQuoteData::mock_contract_call(ROUTER, "1000", "", swap_memo));
            assert_eq!(memo_data_bytes(&TransactionLoadInput { input_type, ..with_memo.clone() }).unwrap(), bytes, "contract swap memo {swap_memo:?}");
        }

        let invalid = TransactionInputType::mock_swap(tron, SwapQuoteData::mock_contract_call(ROUTER, "1000", "", Some("0xzz")));
        assert_eq!(memo_data_bytes(&TransactionLoadInput::mock_tron(invalid, 1000)).unwrap_err().to_string(), "invalid Tron swap memo");
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_USDT_TOKEN_ID, create_test_client};
    use chain_traits::{ChainTraits, ChainTransactionLoad};
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, AssetType, Chain};

    #[tokio::test]
    async fn test_get_transaction_fee_estimates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let estimates = create_test_client().get_transaction_fee_estimates().await?;
        let token_transfer = estimates.token_transfer.unwrap();
        let swap = estimates.swap.unwrap();

        assert_eq!(estimates.fee_asset, AssetId::from_chain(Chain::Tron));
        assert!(estimates.transfer[0].fee > BigInt::from(0));
        assert!(token_transfer[0].fee > estimates.transfer[0].fee);
        assert!(swap[0].fee > token_transfer[0].fee);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_load_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let asset = Asset::from_chain(Chain::Tron);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer { asset });
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

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer { asset });
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
