use crate::jsonrpc::{SolanaAccountEncoding, SolanaProgramAccountsFilter, SolanaRpc, SolanaRpcConfig, SolanaTokenAccountsFilter};
use crate::models::{
    AccountData, EpochInfo, InflationRate, ResultTokenInfo, SupplyResult, TokenAccountInfo, ValueResult, VoteAccounts, balances::SolanaBalance, blockhash::SolanaBlockhashResult,
    prioritization_fee::SolanaPrioritizationFee, simulation::SimulateTransactionResult, transaction::BlockTransactions,
};
use crate::{
    STAKE_PROGRAM_ID,
    metaplex::{decode_metadata, metadata::Metadata},
};
#[cfg(feature = "rpc")]
use gem_client::Client;
use gem_encoding::decode_base64;
#[cfg(feature = "rpc")]
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcError};
use primitives::Chain;
#[cfg(feature = "rpc")]
use serde::de::DeserializeOwned;
use solana_primitives::{AddressLookupTableAccount, Pubkey};
use std::{error::Error, str::FromStr};

#[cfg(feature = "rpc")]
pub struct SolanaClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    pub chain: Chain,
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> SolanaClient<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Solana }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_balance(&self, address: &str) -> Result<SolanaBalance, JsonRpcError> {
        self.client.request(SolanaRpc::GetBalance(address.to_string())).await
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, JsonRpcError> {
        self.client
            .request(SolanaRpc::GetTokenAccountsByOwner(
                owner.to_string(),
                SolanaTokenAccountsFilter::ProgramId(program_id.to_string()),
            ))
            .await
    }

    pub async fn get_epoch_info(&self) -> Result<EpochInfo, JsonRpcError> {
        self.client.request(SolanaRpc::GetEpochInfo(SolanaRpcConfig::Confirmed)).await
    }

    pub async fn get_token_accounts_by_mint(&self, owner: &str, mint: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, JsonRpcError> {
        self.client
            .request(SolanaRpc::GetTokenAccountsByOwner(owner.to_string(), SolanaTokenAccountsFilter::Mint(mint.to_string())))
            .await
    }

    pub async fn get_transaction<T: DeserializeOwned + Send>(&self, signature: &str) -> Result<Option<T>, JsonRpcError> {
        self.client.request(SolanaRpc::GetTransaction(signature.to_string())).await
    }

    pub async fn get_genesis_hash(&self) -> Result<String, JsonRpcError> {
        self.client.request(SolanaRpc::GetGenesisHash).await
    }

    pub async fn get_slot(&self) -> Result<u64, JsonRpcError> {
        self.client.request(SolanaRpc::GetSlot(SolanaRpcConfig::Confirmed)).await
    }

    pub async fn get_latest_blockhash(&self) -> Result<SolanaBlockhashResult, JsonRpcError> {
        self.client.request(SolanaRpc::GetLatestBlockhash(SolanaRpcConfig::Confirmed)).await
    }

    pub async fn get_multiple_accounts(&self, addresses: Vec<String>) -> Result<ValueResult<Vec<Option<AccountData>>>, JsonRpcError> {
        self.client.request(SolanaRpc::GetMultipleAccounts(addresses)).await
    }

    pub async fn get_address_lookup_tables(&self, addresses: Vec<String>) -> Result<Vec<AddressLookupTableAccount>, Box<dyn Error + Send + Sync>> {
        if addresses.is_empty() {
            return Ok(Vec::new());
        }

        let result = self.get_multiple_accounts(addresses.clone()).await?;
        result
            .value
            .into_iter()
            .enumerate()
            .filter_map(|(index, account)| account.map(|account| (index, account)))
            .map(|(index, account)| {
                let data = account
                    .data
                    .first()
                    .ok_or_else(|| -> Box<dyn Error + Send + Sync> { "Missing Solana account data".into() })?;
                let bytes = decode_base64(data)?;
                let address = Pubkey::from_str(&addresses[index])?;
                AddressLookupTableAccount::from_account_data(address, &bytes)
                    .map_err(|err| -> Box<dyn Error + Send + Sync> { format!("Invalid Solana address lookup table: {err}").into() })
            })
            .collect()
    }

    pub async fn get_staking_balance(&self, address: &str) -> Result<Vec<TokenAccountInfo>, JsonRpcError> {
        let filters = vec![SolanaProgramAccountsFilter::Memcmp {
            offset: 12,
            bytes: address.to_string(),
        }];
        self.client.request(SolanaRpc::GetProgramAccounts(STAKE_PROGRAM_ID.to_string(), filters)).await
    }

    pub async fn get_vote_accounts(&self, keep_unstaked_delinquents: bool) -> Result<VoteAccounts, JsonRpcError> {
        self.client.request(SolanaRpc::GetVoteAccounts { keep_unstaked_delinquents }).await
    }

    pub async fn get_inflation_rate(&self) -> Result<InflationRate, JsonRpcError> {
        self.client.request(SolanaRpc::GetInflationRate).await
    }

    pub async fn get_supply(&self) -> Result<SupplyResult, JsonRpcError> {
        self.client.request(SolanaRpc::GetSupply).await
    }

    pub async fn broadcast_transaction(&self, data: String, skip_preflight: Option<bool>) -> Result<String, JsonRpcError> {
        self.client.request(SolanaRpc::SendTransaction { data, skip_preflight }).await
    }

    pub async fn simulate_encoded_transaction(&self, encoded_transaction: &str) -> Result<SimulateTransactionResult, JsonRpcError> {
        let response: ValueResult<SimulateTransactionResult> = self.client.request(SolanaRpc::SimulateTransaction(encoded_transaction.to_string())).await?;
        Ok(response.value)
    }

    pub async fn get_recent_prioritization_fees(&self) -> Result<Vec<SolanaPrioritizationFee>, JsonRpcError> {
        self.client.request(SolanaRpc::GetRecentPrioritizationFees(Vec::new())).await
    }

    pub async fn get_token_mint_info(&self, token_mint: &str) -> Result<ResultTokenInfo, JsonRpcError> {
        self.client
            .request(SolanaRpc::GetAccountInfo(token_mint.to_string(), SolanaAccountEncoding::JsonParsed))
            .await
    }

    pub(crate) async fn get_account_info_base64(&self, address: &str) -> Result<ValueResult<Option<AccountData>>, JsonRpcError> {
        self.client.request(SolanaRpc::GetAccountInfo(address.to_string(), SolanaAccountEncoding::Base64)).await
    }

    pub(crate) async fn find_token_account(&self, owner: &str, mint: &str) -> Result<Option<String>, JsonRpcError> {
        let accounts = self.get_token_accounts_by_mint(owner, mint).await?;
        Ok(accounts.value.first().map(|account| account.pubkey.clone()))
    }

    pub async fn get_metaplex_metadata(&self, token_mint: &str) -> Result<Metadata, Box<dyn Error + Send + Sync>> {
        let pubkey = Pubkey::from_str(token_mint)?;
        let metadata_key = Metadata::find_pda(pubkey)
            .ok_or::<Box<dyn Error + Send + Sync>>("metadata program account not found".into())?
            .0
            .to_string();
        let value = self.get_account_info_base64(&metadata_key).await?.value.ok_or("Failed to get metadata")?;
        let data = value.data.first().ok_or("Missing metadata account data")?;
        decode_metadata(data).map_err(|_| "Failed to decode metadata".into())
    }

    pub async fn get_block_transactions(&self, slot: u64) -> Result<BlockTransactions, JsonRpcError> {
        self.client.request(SolanaRpc::GetBlock(slot)).await
    }

    pub async fn get_token_accounts(&self, address: &str, token_mints: &[String]) -> Result<Vec<ValueResult<Vec<TokenAccountInfo>>>, Box<dyn Error + Send + Sync>> {
        let requests: Vec<SolanaRpc> = token_mints
            .iter()
            .map(|mint| SolanaRpc::GetTokenAccountsByOwner(address.to_string(), SolanaTokenAccountsFilter::Mint(mint.to_string())))
            .collect();
        Ok(self.client.batch_request(requests).await?.take_all()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::ResultTokenInfo;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct JsonRpcResult<T> {
        result: T,
    }

    #[test]
    fn test_decode_token_data() {
        let json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/pyusd_mint.json")).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        assert_eq!(result.result.value.data.parsed.info.decimals, 6);

        let json: serde_json::Value = serde_json::from_str(include_str!("../../testdata/usdc_mint.json")).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        assert_eq!(result.result.value.data.parsed.info.decimals, 6);
    }
}
