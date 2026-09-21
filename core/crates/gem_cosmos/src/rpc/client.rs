use std::error::Error;

use crate::constants::get_base_fee;
use crate::models::account::{Balance, Balances, Page};
use crate::models::staking::{Delegation, Delegations, Rewards, UnbondingDelegation, UnbondingDelegations};
use crate::models::{Account, AccountResponse, BroadcastRequest, BroadcastResponse, InjectiveAccount};
use crate::models::{
    AnnualProvisionsResponse, BaseFeeResponse, BlockResponse, FeemarketGasPriceResponse, InflationResponse, InjectiveBaseFeeResponse, OsmosisEpochProvisionsResponse, OsmosisMintParamsResponse, SmartQueryResponse, StakingPoolResponse,
    SupplyResponse, TransactionResponse, TransactionsResponse, ValidatorsResponse,
};
use crate::provider::state_mapper::map_transfer_fee;
use crate::rpc::target::CosmosTarget;
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainSimulation, ChainTraits};
use gem_client::{Client, ClientExt};
use gem_encoding::encode_base64;
use num_bigint::BigInt;
use primitives::chain_cosmos::CosmosChain;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct CosmosClient<C: Client> {
    pub chain: CosmosChain,
    pub client: C,
}

impl<C: Client> CosmosClient<C> {
    pub fn new(chain: CosmosChain, client: C) -> Self {
        Self { chain, client }
    }

    pub fn get_chain(&self) -> CosmosChain {
        self.chain
    }

    pub async fn get_contract_smart_query<Q: Serialize + Send + Sync, R: DeserializeOwned + Send>(&self, contract: &str, query: &Q) -> Result<R, Box<dyn Error + Send + Sync>> {
        let target = CosmosTarget::GetContractSmartQuery {
            contract: contract.to_string(),
            encoded_query: encode_base64(serde_json::to_string(query)?.as_bytes()),
        };
        Ok(self.client.get::<SmartQueryResponse<R>>(target).await?.data)
    }

    pub async fn get_transaction(&self, hash: String) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetTransaction { hash }).await?)
    }

    pub async fn get_block(&self, block: &str) -> Result<BlockResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetBlock { height: block.to_string() }).await?)
    }

    pub async fn get_transactions_by_address_with_limit(&self, address: &str, limit: usize) -> Result<Vec<TransactionResponse>, Box<dyn Error + Send + Sync>> {
        let key = match self.chain {
            CosmosChain::Cosmos | CosmosChain::Osmosis | CosmosChain::Celestia | CosmosChain::Injective | CosmosChain::Noble => "query",
            CosmosChain::Sei => "events",
            CosmosChain::Thorchain | CosmosChain::Mayachain => return Ok(vec![]),
        };

        let inbound_query = format!("message.sender='{address}'");
        let outbound_query = format!("message.recipient='{address}'");
        let (inbound, outbound) = futures::try_join!(self.get_transactions_by_query(key, &inbound_query, limit), self.get_transactions_by_query(key, &outbound_query, limit),)?;
        let responses = inbound.tx_responses.into_iter().chain(outbound.tx_responses).collect::<Vec<_>>();
        let txs = inbound.txs.into_iter().chain(outbound.txs).collect::<Vec<_>>();
        Ok(responses.into_iter().zip(txs).map(|(response, tx)| TransactionResponse { tx, tx_response: response }).collect::<Vec<_>>())
    }

    pub async fn get_transactions_by_query(&self, key: &'static str, filter: &str, limit: usize) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetTransactions { key, filter: filter.to_string(), limit }).await?)
    }

    pub async fn get_validators(&self) -> Result<ValidatorsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetValidators).await?)
    }

    pub async fn get_delegations_validators(&self, address: &str) -> Result<ValidatorsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetDelegatorValidators { address: address.to_string() }).await?)
    }

    pub async fn get_staking_pool(&self) -> Result<StakingPoolResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetStakingPool).await?)
    }

    pub async fn get_inflation(&self) -> Result<InflationResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetInflation).await?)
    }

    pub async fn get_celestia_annual_provisions(&self) -> Result<AnnualProvisionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetAnnualProvisions).await?)
    }

    pub async fn get_supply_by_denom(&self, denom: &str) -> Result<SupplyResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetSupply { denom: denom.to_string() }).await?)
    }

    pub async fn get_osmosis_mint_params(&self) -> Result<OsmosisMintParamsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetOsmosisMintParams).await?)
    }

    pub async fn get_osmosis_epoch_provisions(&self) -> Result<OsmosisEpochProvisionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetOsmosisEpochProvisions).await?)
    }

    pub async fn get_balances(&self, address: &str) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        self.get_all_pages::<Balances>(|page_key| CosmosTarget::GetBalances { address: address.to_string(), page_key }).await
    }

    async fn get_all_pages<P: Page + DeserializeOwned + Send>(&self, target: impl Fn(Option<String>) -> CosmosTarget) -> Result<Vec<P::Item>, Box<dyn Error + Send + Sync>> {
        let mut items = Vec::new();
        let mut page_key: Option<String> = None;
        loop {
            let requested = page_key.clone();
            let page: P = self.client.get(target(page_key)).await?;
            page_key = page.next_page_key();
            items.extend(page.into_items());
            if page_key.is_none() || page_key == requested {
                return Ok(items);
            }
        }
    }

    pub async fn get_delegations(&self, address: &str) -> Result<Vec<Delegation>, Box<dyn Error + Send + Sync>> {
        self.get_all_pages::<Delegations>(|page_key| CosmosTarget::GetDelegations { address: address.to_string(), page_key }).await
    }

    pub async fn get_unbonding_delegations(&self, address: &str) -> Result<Vec<UnbondingDelegation>, Box<dyn Error + Send + Sync>> {
        self.get_all_pages::<UnbondingDelegations>(|page_key| CosmosTarget::GetUnbondingDelegations { address: address.to_string(), page_key }).await
    }

    pub async fn get_delegation_rewards(&self, address: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetDelegationRewards { address: address.to_string() }).await?)
    }

    pub async fn get_base_fee(&self) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
        Ok(match self.chain {
            CosmosChain::Cosmos => {
                let denom = self.chain.denom().as_ref().to_string();
                let response: FeemarketGasPriceResponse = self.client.get(CosmosTarget::GetFeemarketGasPrice { denom }).await?;
                map_transfer_fee(&response.price.amount)?
            }
            CosmosChain::Osmosis => {
                let response: BaseFeeResponse = self.client.get(CosmosTarget::GetBaseFee { chain: self.chain }).await?;
                map_transfer_fee(&response.base_fee)?
            }
            CosmosChain::Injective => {
                let response: InjectiveBaseFeeResponse = self.client.get(CosmosTarget::GetBaseFee { chain: self.chain }).await?;
                map_transfer_fee(&response.base_fee.base_fee)?
            }
            CosmosChain::Celestia | CosmosChain::Sei | CosmosChain::Noble | CosmosChain::Thorchain | CosmosChain::Mayachain => BigInt::from(get_base_fee(self.chain)),
        })
    }

    pub async fn get_account_info(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let target = CosmosTarget::GetAccount { address: address.to_string() };
        match self.chain {
            CosmosChain::Injective => Ok(self.client.get::<AccountResponse<InjectiveAccount>>(target).await?.account.base_account),
            _ => Ok(self.client.get::<AccountResponse<Account>>(target).await?.account),
        }
    }

    pub async fn get_node_info(&self) -> Result<crate::models::NodeInfoResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(CosmosTarget::GetNodeInfo).await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<BroadcastResponse, Box<dyn Error + Send + Sync>> {
        let request: BroadcastRequest = serde_json::from_str(data)?;
        Ok(self.client.post(CosmosTarget::BroadcastTransaction, &request).await?)
    }
}

impl<C: Client> ChainAccount for CosmosClient<C> {}

impl<C: Client> ChainPerpetual for CosmosClient<C> {}

impl<C: Client> ChainAddressStatus for CosmosClient<C> {}

impl<C: Client> ChainSimulation for CosmosClient<C> {}

impl<C: Client> ChainTraits for CosmosClient<C> {}

impl<C: Client> chain_traits::ChainProvider for CosmosClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain.as_chain()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_celestia_transactions_by_address() {
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_client = request_count.clone();
        let client = MockClient::new().with_get(move |path| {
            let address = "celestia1cvh8mpz04az0x7vht6h6ekksg8wd650rq0wm5l";
            let expected_path = match request_count_for_client.fetch_add(1, Ordering::SeqCst) {
                0 => format!("/cosmos/tx/v1beta1/txs?query=message.sender='{address}'&pagination.limit=1&page=1"),
                1 => format!("/cosmos/tx/v1beta1/txs?query=message.recipient='{address}'&pagination.limit=1&page=1"),
                _ => panic!("unexpected request"),
            };
            assert_eq!(path, expected_path);
            Ok(include_str!("../../testdata/empty_transactions.json").as_bytes().to_vec())
        });

        let transactions = CosmosClient::new(CosmosChain::Celestia, client)
            .get_transactions_by_address_with_limit("celestia1cvh8mpz04az0x7vht6h6ekksg8wd650rq0wm5l", 1)
            .await
            .unwrap();

        assert_eq!(transactions.len(), 0);
        assert_eq!(request_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_balances_are_read_until_the_last_page() {
        let requested = Arc::new(std::sync::Mutex::new(Vec::new()));
        let requested_by_client = requested.clone();
        let client = MockClient::new().with_get(move |path| {
            requested_by_client.lock().unwrap().push(path.to_string());
            let body = match path.contains("pagination.key") {
                false => r#"{"balances":[{"denom":"uatom","amount":"1"}],"pagination":{"next_key":"cGFnZSB0d28="}}"#,
                true => r#"{"balances":[{"denom":"uosmo","amount":"2"}],"pagination":{"next_key":null}}"#,
            };
            Ok(body.as_bytes().to_vec())
        });

        let balances = CosmosClient::new(CosmosChain::Cosmos, client).get_balances("cosmos1").await.unwrap();

        assert_eq!(balances.iter().map(|balance| balance.denom.as_str()).collect::<Vec<_>>(), vec!["uatom", "uosmo"]);
        assert_eq!(
            *requested.lock().unwrap(),
            vec!["/cosmos/bank/v1beta1/balances/cosmos1".to_string(), "/cosmos/bank/v1beta1/balances/cosmos1?pagination.key=cGFnZSB0d28%3D".to_string()]
        );
    }

    #[tokio::test]
    async fn test_a_page_that_cannot_be_read_fails_the_whole_read() {
        let client = MockClient::new().with_get(move |path| match path.contains("pagination.key") {
            false => Ok(r#"{"balances":[{"denom":"uatom","amount":"1"}],"pagination":{"next_key":"cGFnZSB0d28="}}"#.as_bytes().to_vec()),
            true => Err(gem_client::ClientError::Network("page two is gone".to_string())),
        });

        let result = CosmosClient::new(CosmosChain::Cosmos, client).get_balances("cosmos1").await;

        assert!(result.is_err(), "a partial read must not be published as the whole balance");
    }

    #[tokio::test]
    async fn test_a_cursor_that_never_advances_stops_the_read() {
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_client = request_count.clone();
        let client = MockClient::new().with_get(move |_| {
            request_count_for_client.fetch_add(1, Ordering::SeqCst);
            Ok(r#"{"balances":[{"denom":"uatom","amount":"1"}],"pagination":{"next_key":"c3R1Y2s="}}"#.as_bytes().to_vec())
        });

        let balances = CosmosClient::new(CosmosChain::Cosmos, client).get_balances("cosmos1").await.unwrap();

        assert_eq!(balances.len(), 2);
        assert_eq!(request_count.load(Ordering::SeqCst), 2, "a repeated cursor ends the read instead of looping forever");
    }
}
