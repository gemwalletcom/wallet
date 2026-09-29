use std::collections::HashSet;
use std::str::FromStr;

use alchemy::rpc::{Client as AlchemyClient, INTERNAL_TRANSFER_CHAINS, Transfer, TransferCategory};
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::{
    ethereum_address_checksum,
    rpc::{EthereumClient, EthereumProvider},
};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{
    AssetId, Chain, EVMChain, SwapProvider, Transaction, TransactionIdRequest, TransactionState,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};
use serde::{Deserialize, Serialize};

use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const TRANSFERS_LIMIT: usize = 100;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct EvmPartnerCursor {
    block: u64,
    page_key: Option<String>,
}

pub struct EvmPartnerProvider<C: Client + Clone> {
    chain: Chain,
    alchemy: AlchemyClient<C>,
    provider: EthereumProvider<C>,
    fee_address: String,
}

impl<C: Client + Clone + 'static> EvmPartnerProvider<C> {
    pub fn new(chain: EVMChain, alchemy: JsonRpcClient<C>, node: JsonRpcClient<C>) -> Self {
        Self {
            chain: chain.to_chain(),
            alchemy: AlchemyClient::new(alchemy),
            provider: EthereumProvider::new_rpc_only(EthereumClient::new(node, chain)),
            fee_address: default_referral_fees().evm.address,
        }
    }

    fn transfer_categories(&self) -> Vec<TransferCategory> {
        if INTERNAL_TRANSFER_CHAINS.contains(&self.chain) {
            vec![TransferCategory::External, TransferCategory::Internal, TransferCategory::Erc20]
        } else {
            vec![TransferCategory::External, TransferCategory::Erc20]
        }
    }

    async fn get_partner_transaction(&self, transfer: &Transfer) -> Result<Option<SwapPartnerTransaction>, SwapperError> {
        let request = TransactionIdRequest::new(self.chain, transfer.hash.clone(), Some(transfer.block_num));
        let transaction = self.provider.get_transaction_with_receipt(request).await.map_err(SwapperError::compute_quote_error)?;
        Ok(transaction.and_then(|(transaction, _)| map_partner_transaction(self.chain, &transaction, transfer)))
    }
}

fn is_router_provider(provider: SwapProvider) -> bool {
    match provider {
        SwapProvider::UniswapV3 | SwapProvider::UniswapV4 | SwapProvider::PancakeswapV3 | SwapProvider::Aerodrome | SwapProvider::Oku | SwapProvider::Wagmi | SwapProvider::Okx => true,
        SwapProvider::Panora
        | SwapProvider::Thorchain
        | SwapProvider::Jupiter
        | SwapProvider::Across
        | SwapProvider::StonfiV2
        | SwapProvider::Mayan
        | SwapProvider::Chainflip
        | SwapProvider::NearIntents
        | SwapProvider::CetusClmm
        | SwapProvider::Relay
        | SwapProvider::Hyperliquid
        | SwapProvider::Orca
        | SwapProvider::Squid
        | SwapProvider::Mayachain
        | SwapProvider::SwapsXyz => false,
    }
}

fn map_fee_asset_id(chain: Chain, transfer: &Transfer) -> Option<AssetId> {
    match transfer.category {
        TransferCategory::External | TransferCategory::Internal => Some(AssetId::from_chain(chain)),
        TransferCategory::Erc20 => Some(AssetId::from_token(chain, &ethereum_address_checksum(transfer.raw_contract.address.as_ref()?).ok()?)),
        TransferCategory::Erc721 | TransferCategory::Erc1155 | TransferCategory::Specialnft => None,
    }
}

pub fn map_partner_transaction(chain: Chain, transaction: &Transaction, fee: &Transfer) -> Option<SwapPartnerTransaction> {
    let swap = transaction.swap_metadata()?;
    let provider = SwapProvider::from_str(swap.provider.as_deref()?).ok()?;
    if !is_router_provider(provider) {
        return None;
    }
    Some(SwapPartnerTransaction {
        provider,
        provider_transaction_id: transaction.id.hash.clone(),
        status: if transaction.state == TransactionState::Confirmed { SwapStatus::Completed } else { SwapStatus::Failed },
        from_address: transaction.from.clone(),
        to_address: transaction.to.clone(),
        from_asset_id: swap.from_asset,
        from_value: swap.from_value.to_string(),
        from_amount_usd: None,
        to_asset_id: swap.to_asset,
        to_value: swap.to_value.to_string(),
        to_amount_usd: None,
        referral_fee: Some(SwapReferralFee {
            asset_id: map_fee_asset_id(chain, fee)?,
            value: fee.raw_contract.value.as_ref()?.to_string(),
            amount_usd: None,
        }),
        from_transaction_hash: Some(transaction.id.hash.clone()),
        to_transaction_hash: Some(transaction.id.hash.clone()),
    })
}

fn first_transfer_per_transaction(transfers: Vec<Transfer>) -> Vec<Transfer> {
    let mut hashes = HashSet::new();
    transfers.into_iter().filter(|transfer| hashes.insert(transfer.hash.clone())).collect()
}

#[async_trait]
impl<C: Client + Clone + 'static> SwapPartnerProvider for EvmPartnerProvider<C> {
    fn name(&self) -> &str {
        self.chain.as_ref()
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor: EvmPartnerCursor = cursor.filter(|cursor| !cursor.is_empty()).map(|cursor| serde_json::from_str(&cursor)).transpose()?.unwrap_or_default();
        let page = self
            .alchemy
            .get_transfers_to(&self.fee_address, &self.transfer_categories(), cursor.block, cursor.page_key.clone(), TRANSFERS_LIMIT)
            .await
            .map_err(SwapperError::compute_quote_error)?;
        let last_block = page.transfers.last().map_or(cursor.block, |transfer| transfer.block_num);
        let mut transactions = Vec::new();
        for transfer in first_transfer_per_transaction(page.transfers) {
            transactions.extend(self.get_partner_transaction(&transfer).await?);
        }
        let next = match page.page_key {
            Some(page_key) => SwapPartnerCursor::Next(serde_json::to_string(&EvmPartnerCursor {
                block: cursor.block,
                page_key: Some(page_key),
            })?),
            None => SwapPartnerCursor::Latest(serde_json::to_string(&EvmPartnerCursor { block: last_block, page_key: None })?),
        };
        Ok(SwapPartnerTransactionsPage { transactions, cursor: next })
    }
}
