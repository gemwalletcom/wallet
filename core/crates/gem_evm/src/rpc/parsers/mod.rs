pub mod across;
pub mod mayan;
pub mod okx;
pub mod pancakeswap;
pub mod staking;
pub mod universal_router;
pub mod yo;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::Num;

use super::model::{Transaction, TransactionReceipt};
use crate::ethereum_address_checksum;
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionType};

use self::{
    across::AcrossParser, mayan::MayanParser, okx::OkxParser, pancakeswap::PancakeSwapParser, staking::MonadStakingParser, staking::SmartChainStakingParser,
    universal_router::UniversalRouterParser, yo::YoParser,
};

pub const EVENT_WORD_SIZE: usize = 64;

pub struct ParseContext<'a> {
    pub chain: &'a Chain,
    pub transaction: &'a Transaction,
    pub receipt: &'a TransactionReceipt,
    pub created_at: DateTime<Utc>,
}

impl ParseContext<'_> {
    pub fn is_to_any(&self, addresses: &[&str]) -> bool {
        self.transaction
            .to
            .as_ref()
            .is_some_and(|to| addresses.iter().any(|address| to.eq_ignore_ascii_case(address)))
    }

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
}

pub trait ProtocolParser: Send + Sync {
    fn matches(&self, context: &ParseContext<'_>) -> bool;
    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction>;
}

pub fn ethereum_value_from_log_data(data: &str, start: usize, end: usize) -> Option<BigUint> {
    data.trim_start_matches("0x").get(start..end).and_then(|s| BigUint::from_str_radix(s, 16).ok())
}

pub struct ProtocolParsers;

impl ProtocolParsers {
    fn default_parsers() -> [&'static dyn ProtocolParser; 8] {
        [
            &MonadStakingParser,
            &SmartChainStakingParser,
            &AcrossParser,
            &MayanParser,
            &OkxParser,
            &YoParser,
            &PancakeSwapParser,
            &UniversalRouterParser,
        ]
    }

    pub fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt, created_at: DateTime<Utc>) -> Option<PrimitivesTransaction> {
        Self::map_transaction_with_parsers(chain, transaction, receipt, created_at, &[])
    }

    pub fn map_transaction_with_parsers(
        chain: &Chain,
        transaction: &Transaction,
        receipt: &TransactionReceipt,
        created_at: DateTime<Utc>,
        parsers: &[&'static dyn ProtocolParser],
    ) -> Option<PrimitivesTransaction> {
        let context = ParseContext {
            chain,
            transaction,
            receipt,
            created_at,
        };

        parsers
            .iter()
            .copied()
            .chain(Self::default_parsers())
            .filter(|parser| parser.matches(&context))
            .find_map(|parser| parser.parse(&context))
    }
}
