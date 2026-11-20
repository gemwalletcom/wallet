use alloy_sol_types::SolCall;
use primitives::hex;
use primitives::{Asset, AssetId, asset_type::AssetType, chain::Chain, decode_hex};
use std::error::Error;

use crate::models::{
    Block, BlockId, BlockTransactions, BlockTransactionsInfo, ChainParameter, ChainParametersResponse, NowBlockRequest, Transaction, TransactionReceiptData, TriggerConstantContractRequest, TriggerConstantContractResponse,
    TriggerSmartContractRequest, TronTransactionBroadcast, WitnessesList,
};
use crate::models::{TriggerSmartContractData, TronAccount, TronAccountRequest, TronAccountUsage, TronBlock, TronContractInfo, TronContractRequest, TronEmptyAccount, TronReward};
use crate::rpc::constants::{DECIMALS_SELECTOR, DEFAULT_OWNER_ADDRESS, GENESIS_BLOCK_NUMBER, NAME_SELECTOR, SYMBOL_SELECTOR};
use crate::rpc::target::TronTarget;
use gem_client::{Client, ClientExt};
use gem_evm::contracts::erc20::decode_token_metadata;
use serde::de::DeserializeOwned;

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

    pub async fn get_transaction(&self, id: String) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_unless_empty(TronTarget::GetTransactionById { id }).await
    }

    pub async fn get_transaction_receipt(&self, id: String) -> Result<Option<TransactionReceiptData>, Box<dyn Error + Send + Sync>> {
        self.get_unless_empty(TronTarget::GetTransactionInfoById { id }).await
    }

    async fn get_unless_empty<T: DeserializeOwned>(&self, target: TronTarget) -> Result<Option<T>, Box<dyn Error + Send + Sync>> {
        let response: serde_json::Value = self.client.get(target).await?;
        if response.as_object().is_some_and(serde_json::Map::is_empty) {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(response)?))
    }

    pub async fn call_contract<T: SolCall + Sync>(&self, contract_address: &str, call: &T) -> Result<T::Return, Box<dyn Error + Send + Sync>> {
        let result = self.trigger_constant_contract(contract_address, T::SIGNATURE, &hex::encode(&call.abi_encode()[4..])).await?;
        Ok(T::abi_decode_returns(&decode_hex(&result)?)?)
    }

    pub async fn trigger_constant_contract(&self, contract_address: &str, function_selector: &str, parameter: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = TriggerConstantContractRequest {
            owner_address: DEFAULT_OWNER_ADDRESS.to_owned(),
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
        let response: TriggerConstantContractResponse = self.client.post(TronTarget::TriggerConstantContract, request).await?;
        Ok(response.check_result()?)
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

        let (name, symbol, decimals) = decode_token_metadata(&name, &symbol, &decimals)?;
        let asset_id = AssetId::from(Chain::Tron, Some(token_id));
        Ok(Asset::new(asset_id, name, symbol, u32::from(decimals), AssetType::TRC20))
    }

    pub async fn get_account(&self, address: &str) -> Result<TronAccount, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest { address: address.to_string(), visible: true };

        Ok(self.client.post(TronTarget::GetAccount, &request).await?)
    }

    pub async fn get_account_usage(&self, address: &str) -> Result<TronAccountUsage, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest { address: address.to_string(), visible: true };

        Ok(self.client.post(TronTarget::GetAccountResource, &request).await?)
    }

    pub async fn get_contract(&self, address: &str) -> Result<TronContractInfo, Box<dyn Error + Send + Sync>> {
        let request = TronContractRequest { value: address.to_string(), visible: true };

        Ok(self.client.post(TronTarget::GetContract, &request).await?)
    }

    pub async fn get_reward(&self, address: &str) -> Result<TronReward, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest { address: address.to_string(), visible: true };

        Ok(self.client.post(TronTarget::GetReward, &request).await?)
    }

    pub async fn is_new_account(&self, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let request = TronAccountRequest { address: address.to_string(), visible: true };

        let account: TronEmptyAccount = self.client.post(TronTarget::GetAccount, &request).await?;
        Ok(account.address.is_none_or(|address| address.is_empty()))
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<TronTransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let json_value: serde_json::Value = serde_json::from_str(&data)?;
        Ok(self.client.post(TronTarget::BroadcastTransaction, &json_value).await?)
    }

    pub async fn get_tron_block(&self) -> Result<TronBlock, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(TronTarget::GetNowBlock, &NowBlockRequest {}).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_estimate_energy_with_data() {
        let contract_data = TriggerSmartContractData {
            contract_address: "Tusdt".to_string(),
            data: "a9059cbb".to_string(),
            owner_address: "Tsender".to_string(),
            fee_limit: None,
            call_value: None,
        };

        let mock = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/trigger_constant_contract_with_penalty.json").as_bytes().to_vec()));
        let energy = TronClient::new(mock).estimate_energy_with_data(&contract_data).await.unwrap();
        assert_eq!(energy, 64285);

        let mock = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/trigger_constant_contract_failed.json").as_bytes().to_vec()));
        let error = TronClient::new(mock).estimate_energy_with_data(&contract_data).await.unwrap_err().to_string();
        assert!(error.contains("CONTRACT_VALIDATE_ERROR"), "expected structured Tron RPC error, got: {error}");
    }
}
