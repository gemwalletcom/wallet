use std::time::Duration;

use super::ToolFailure;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use strum::{Display, EnumIter};

use crate::tools::enum_slugs;

const BASE_URL: &str = "https://api.gemwallet.com";

#[derive(Clone)]
pub struct GemApiTool {
    pub client: reqwest::Client,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum GemApiAction {
    Transaction,
    AddressBalances,
    AddressAssets,
    AddressTransactions,
    AddressNfts,
    StakingValidators,
    StakingApy,
    Asset,
    Nft,
    NftCollection,
}

#[derive(Debug, Deserialize)]
pub struct GemApiArgs {
    pub action: GemApiAction,
    #[serde(default)]
    pub chain: Option<String>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub from_timestamp: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct GemApiOutput(Value);

impl Tool for GemApiTool {
    const NAME: &'static str = "gem_api";
    type Error = ToolFailure;
    type Args = GemApiArgs;
    type Output = GemApiOutput;

    async fn definition(&self, _: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Query the public Gem Wallet API at api.gemwallet.com (same backend the \
                apps use, normalized across chains) for diagnostic checks — faster than per-chain \
                explorers. It's Gem's indexer: if it disagrees with the chain, the chain wins \
                (flag the gap), and for load-bearing claims the on-chain receipt is canonical. \
                <chain> uses Gem's canonical chain ids (see chains.md); <asset_id> is the canonical \
                asset id (ethereum, ethereum_0x… for ERC-20s, solana_… for SPL). action: \
                transaction (full tx record, needs chain + hash); \
                address_balances (native + staking-with-delegation + assets in one payload, needs chain + address); \
                address_assets (token balances only, needs chain + address); \
                address_transactions (recent txs, needs chain + address, optional from_timestamp unix seconds); \
                address_nfts (owned NFTs, needs chain + address); \
                staking_validators (needs chain); staking_apy (needs chain); \
                asset (metadata/price/supply, needs asset_id); \
                nft (needs asset_id); nft_collection (needs collection_id)."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": enum_slugs::<GemApiAction>() },
                    "chain": { "type": "string" },
                    "hash": { "type": "string" },
                    "address": { "type": "string" },
                    "asset_id": { "type": "string" },
                    "collection_id": { "type": "string" },
                    "from_timestamp": { "type": "integer", "description": "Unix seconds lower bound for action=address_transactions." }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        use GemApiAction::*;
        let missing = |field: &str| ToolFailure::missing(field, args.action);
        let chain = || args.chain.as_deref().ok_or_else(|| missing("chain"));
        let address = || args.address.as_deref().ok_or_else(|| missing("address"));
        let url = match args.action {
            Transaction => {
                let hash = args.hash.as_deref().ok_or_else(|| missing("hash"))?;
                format!("{BASE_URL}/v1/chain/transactions/{}/{hash}", chain()?)
            }
            AddressBalances => {
                format!("{BASE_URL}/v1/chain/address/{}/{}/balances", chain()?, address()?)
            }
            AddressAssets => {
                format!("{BASE_URL}/v1/chain/address/{}/{}/assets", chain()?, address()?)
            }
            AddressTransactions => {
                let mut url = format!("{BASE_URL}/v1/chain/address/{}/{}/transactions", chain()?, address()?);
                if let Some(ts) = args.from_timestamp {
                    url.push_str(&format!("?from_timestamp={ts}"));
                }
                url
            }
            AddressNfts => {
                format!("{BASE_URL}/v1/chain/address/{}/{}/nfts", chain()?, address()?)
            }
            StakingValidators => {
                format!("{BASE_URL}/v1/chain/staking/{}/validators", chain()?)
            }
            StakingApy => {
                format!("{BASE_URL}/v1/chain/staking/{}/apy", chain()?)
            }
            Asset => {
                let id = args.asset_id.as_deref().ok_or_else(|| missing("asset_id"))?;
                format!("{BASE_URL}/v1/assets/{id}")
            }
            Nft => {
                let id = args.asset_id.as_deref().ok_or_else(|| missing("asset_id"))?;
                format!("{BASE_URL}/v1/chain/nft/assets/{id}")
            }
            NftCollection => {
                let id = args.collection_id.as_deref().ok_or_else(|| missing("collection_id"))?;
                format!("{BASE_URL}/v1/chain/nft/collections/{id}")
            }
        };
        let resp = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| ToolFailure::other(format!("{url}: {e}")))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| ToolFailure::other(format!("{url}: read body: {e}")))?;
        if !status.is_success() {
            return Err(ToolFailure::other(format!("{url} failed: {status}: {body}")));
        }
        let value: Value = serde_json::from_str(&body).map_err(|e| ToolFailure::other(format!("{url} returned non-json: {e}: {body}")))?;
        Ok(GemApiOutput(value))
    }
}
