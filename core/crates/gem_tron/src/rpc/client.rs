use num_bigint::BigUint;
use primitives::{Asset, AssetId, asset_type::AssetType, chain::Chain};
use std::{error::Error, str::FromStr};

use crate::models::{
    Block, BlockId, BlockTransactions, BlockTransactionsInfo, ChainParameter, ChainParametersResponse, NowBlockRequest, Transaction, TransactionReceiptData,
    TriggerConstantContractRequest, TriggerConstantContractResponse, TriggerSmartContractRequest, TronTransactionBroadcast, WitnessesList,
};
use crate::models::{TriggerSmartContractData, TronAccount, TronAccountRequest, TronAccountUsage, TronBlock, TronEmptyAccount, TronReward};
use crate::rpc::constants::{DECIMALS_SELECTOR, DEFAULT_OWNER_ADDRESS, GENESIS_BLOCK_NUMBER, NAME_SELECTOR, SYMBOL_SELECTOR};
use crate::rpc::target::TronTarget;
use gem_client::{Client, ClientExt};
use gem_evm::contracts::erc20::{decode_abi_string, decode_abi_uint8};

#[derive(Clone)]
pub struct TronClient<C: Client> {
    pub client: C,
}

impl<C: Client> TronClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TronTarget::GetBlock).await?)
    }

    pub async fn get_block_transactions(&self, block: u64) -> Result<BlockTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TronTarget::GetBlockByNumber { number: block }).await?)
    }

    pub async fn get_block_transactions_receipts(&self, block: u64) -> Result<BlockTransactionsInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TronTarget::GetTransactionInfoByBlockNumber { number: block }).await?)
    }

    pub async fn get_transaction(&self, id: String) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TronTarget::GetTransactionById { id }).await?)
    }

    pub async fn get_transaction_receipt(&self, id: String) -> Result<Option<TransactionReceiptData>, Box<dyn Error + Send + Sync>> {
        let response: serde_json::Value = self.client.get(TronTarget::GetTransactionInfoById { id }).await?;
        if response.as_object().is_some_and(|object| object.is_empty()) {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(response)?))
    }

    pub async fn trigger_constant_contract(&self, contract_address: &str, function_selector: &str, parameter: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.trigger_constant_contract_with_owner(DEFAULT_OWNER_ADDRESS, contract_address, function_selector, parameter)
            .await
    }

    pub async fn trigger_constant_contract_with_owner(
        &self,
        owner_address: &str,
        contract_address: &str,
        function_selector: &str,
        parameter: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = TriggerConstantContractRequest {
            owner_address: owner_address.to_owned(),
            contract_address: contract_address.to_string(),
            function_selector: function_selector.to_string(),
            parameter: parameter.to_string(),
            fee_limit: None,
            call_value: None,
            visible: true,
        };

        let response = self.trigger_constant_contract_request(&request).await?;

        if response.constant_result.is_empty() {
            return Err("Empty response from Tron contract call".into());
        }

        Ok(response.constant_result[0].clone())
    }

    async fn trigger_constant_contract_request(&self, request: &(impl serde::Serialize + Send + Sync)) -> Result<TriggerConstantContractResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(TronTarget::TriggerConstantContract, request).await?)
    }

    pub async fn estimate_energy_with_data(&self, contract_data: &TriggerSmartContractData) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.trigger_smart_contract_call(contract_data).await?.get_energy()?)
    }

    pub(crate) async fn trigger_smart_contract_call(&self, contract_data: &TriggerSmartContractData) -> Result<TriggerConstantContractResponse, Box<dyn Error + Send + Sync>> {
        let request = TriggerSmartContractRequest {
            owner_address: contract_data.owner_address.clone(),
            contract_address: contract_data.contract_address.clone(),
            data: contract_data.data.clone(),
            fee_limit: contract_data.fee_limit,
            call_value: contract_data.call_value,
            visible: true,
        };

        self.trigger_constant_contract_request(&request).await
    }
}

impl<C: Client> TronClient<C> {
    pub fn get_chain(&self) -> Chain {
        Chain::Tron
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_block().await?.block_header.raw_data.number)
    }

    pub async fn get_genesis_block_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get::<BlockId>(TronTarget::GetBlockByNumber { number: GENESIS_BLOCK_NUMBER }).await?.block_id)
    }

    pub async fn get_witnesses_list(&self) -> Result<WitnessesList, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TronTarget::ListWitnesses).await?)
    }

    pub async fn get_chain_parameters(&self) -> Result<Vec<ChainParameter>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get::<ChainParametersResponse>(TronTarget::GetChainParameters).await?.chain_parameter)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let (name, symbol, decimals) = futures::try_join!(
            self.trigger_constant_contract(&token_id, NAME_SELECTOR, ""),
            self.trigger_constant_contract(&token_id, SYMBOL_SELECTOR, ""),
            self.trigger_constant_contract(&token_id, DECIMALS_SELECTOR, ""),
        )?;

        let name = decode_abi_string(&name)?;
        let symbol = decode_abi_string(&symbol)?;
        let decimals = decode_abi_uint8(&decimals)?;
        let asset_id = AssetId::from(Chain::Tron, Some(token_id));
        Ok(Asset::new(asset_id, name, symbol, decimals as i32, AssetType::TRC20))
    }

    pub async fn get_account(&self, address: &str) -> Result<TronAccount, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post(TronTarget::GetAccount, &request).await?)
    }

    pub async fn get_account_usage(&self, address: &str) -> Result<TronAccountUsage, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post(TronTarget::GetAccountResource, &request).await?)
    }

    pub async fn get_reward(&self, address: &str) -> Result<TronReward, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        Ok(self.client.post(TronTarget::GetReward, &request).await?)
    }

    pub async fn is_new_account(&self, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest {
            address: address.to_string(),
            visible: true,
        };

        let account: TronEmptyAccount = self.client.post(TronTarget::GetAccount, &request).await?;
        Ok(account.address.is_none_or(|addr| addr.is_empty()))
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<TronTransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let json_value: serde_json::Value = serde_json::from_str(&data)?;
        Ok(self.client.post(TronTarget::BroadcastTransaction, &json_value).await?)
    }

    pub async fn get_tron_block(&self) -> Result<TronBlock, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(TronTarget::GetNowBlock, &NowBlockRequest {}).await?)
    }

    pub async fn estimate_trc20_transfer_gas(
        &self,
        sender_address: String,
        contract_address: String,
        recipient_address: String,
        value: String,
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let value_bigint = BigUint::from_str(&value).map_err(|e| format!("Failed to parse value as decimal: {}", e))?;
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));
        let parameter = format!("{}{}", recipient_address, value_hex);

        let request = TriggerConstantContractRequest {
            owner_address: sender_address,
            contract_address,
            function_selector: "transfer(address,uint256)".to_string(),
            parameter,
            fee_limit: None,
            call_value: None,
            visible: true,
        };

        Ok(self.trigger_constant_contract_request(&request).await?.get_energy()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_estimate_trc20_transfer_gas_uses_total_energy_and_surfaces_errors() {
        let mock = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/trigger_constant_contract_with_penalty.json").as_bytes().to_vec()));
        let client = TronClient::new(mock);
        let energy = client
            .estimate_trc20_transfer_gas("Tsender".to_string(), "Tusdt".to_string(), "0".repeat(64), "1000000".to_string())
            .await
            .unwrap();
        assert_eq!(energy, 64285);

        let mock = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/trigger_constant_contract_failed.json").as_bytes().to_vec()));
        let client = TronClient::new(mock);
        let error = client
            .estimate_trc20_transfer_gas("Tsender".to_string(), "Tusdt".to_string(), "0".repeat(64), "1000000".to_string())
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("CONTRACT_VALIDATE_ERROR"), "expected structured Tron RPC error, got: {error}");
    }

    #[test]
    fn test_value_encoding_for_trc20_transfer() {
        let value = "1000000".to_string(); // 1 USDT (6 decimals)
        let recipient_address = "0000000000000000000000003e1451cdb84d440345de6195b0384d1b77aa4eaa".to_string();

        let value_bigint = BigUint::from_str(&value).unwrap();
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));
        let parameter = format!("{}{}", recipient_address, value_hex);

        // For 1000000 (decimal), the hex should be f4240 padded to 64 chars
        assert_eq!(value_hex, "00000000000000000000000000000000000000000000000000000000000f4240");
        assert_eq!(
            parameter,
            "0000000000000000000000003e1451cdb84d440345de6195b0384d1b77aa4eaa00000000000000000000000000000000000000000000000000000000000f4240"
        );
    }

    #[test]
    fn test_large_value_encoding() {
        let value = "16777216".to_string(); // Large value that was causing issues

        let value_bigint = BigUint::from_str(&value).unwrap();
        let value_hex = format!("{:0>64}", hex::encode(value_bigint.to_bytes_be()));

        // 16777216 decimal = 0x1000000 hex
        assert_eq!(value_hex, "0000000000000000000000000000000000000000000000000000000001000000");
    }
}
