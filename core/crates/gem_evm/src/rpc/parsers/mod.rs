pub mod across;
pub mod dex;
pub mod okx;
pub mod pancakeswap;
pub mod staking;
pub mod universal_router;
pub mod yo;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::Num;

use super::{
    balance_differ::BalanceDiffer,
    model::{Transaction, TransactionReceipt, TransactionReplayTrace},
};
use crate::{ethereum_address_checksum, registry::ContractRegistry};
use chain_primitives::SwapMapper as BalanceSwapMapper;
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionType};

use self::{
    across::AcrossParser,
    dex::DexSwapParser,
    okx::OkxParser,
    pancakeswap::PancakeSwapParser,
    staking::{EverstakeParser, MonadStakingParser, SmartChainStakingParser},
    universal_router::UniversalRouterParser,
    yo::YoParser,
};

pub struct ParseContext<'a> {
    pub chain: &'a Chain,
    pub transaction: &'a Transaction,
    pub receipt: &'a TransactionReceipt,
    pub trace: Option<&'a TransactionReplayTrace>,
    pub contract_registry: Option<&'a ContractRegistry>,
    pub created_at: DateTime<Utc>,
}

impl ParseContext<'_> {
    fn make_swap_transaction(&self, from: &str, to: &str, metadata: &TransactionSwapMetadata) -> Option<PrimitivesTransaction> {
        let from = ethereum_address_checksum(from).ok()?;
        let to = ethereum_address_checksum(to).ok()?;
        let contract = self.transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());

        Some(PrimitivesTransaction::new(
            self.transaction.hash.clone(),
            metadata.from_asset.clone(),
            from,
            to,
            contract,
            TransactionType::Swap,
            self.receipt.get_state(),
            self.receipt.get_fee().to_string(),
            AssetId::from_chain(*self.chain),
            self.transaction.value.to_string(),
            None,
            serde_json::to_value(metadata).ok(),
            self.created_at,
        ))
    }

    fn try_map_balance_diff_swap(&self, from: &str, provider: Option<String>) -> Option<TransactionSwapMetadata> {
        let trace = self.trace?;
        let from = ethereum_address_checksum(from).ok()?;
        let differ = BalanceDiffer::new(*self.chain);
        let diff_map = differ.calculate(trace, self.receipt);
        let diff = diff_map.get(&from)?;
        let native_asset_id = self.chain.as_asset_id();
        let fee = self.receipt.get_fee();

        BalanceSwapMapper::map_swap(diff, &fee, &native_asset_id, provider)
    }
}

pub trait ProtocolParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool;
    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction>;
}

fn ethereum_value_from_log_data(data: &str, start: usize, end: usize) -> Option<BigUint> {
    data.trim_start_matches("0x").get(start..end).and_then(|s| BigUint::from_str_radix(s, 16).ok())
}

pub struct ProtocolParsers;

impl ProtocolParsers {
    fn parsers() -> [&'static dyn ProtocolParser; 9] {
        [
            &EverstakeParser,
            &MonadStakingParser,
            &SmartChainStakingParser,
            &AcrossParser,
            &OkxParser,
            &YoParser,
            &PancakeSwapParser,
            &UniversalRouterParser,
            &DexSwapParser,
        ]
    }

    pub fn map_transaction(
        chain: &Chain,
        transaction: &Transaction,
        receipt: &TransactionReceipt,
        trace: Option<&TransactionReplayTrace>,
        contract_registry: Option<&ContractRegistry>,
        created_at: DateTime<Utc>,
    ) -> Option<PrimitivesTransaction> {
        let context = ParseContext {
            chain,
            transaction,
            receipt,
            trace,
            contract_registry,
            created_at,
        };

        Self::parsers()
            .into_iter()
            .filter(|parser| parser.matches(&context))
            .find_map(|parser| parser.parse(&context))
    }
}
