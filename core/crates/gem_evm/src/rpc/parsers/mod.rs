pub mod across;
pub mod mayan;
pub mod okx;
pub mod pancakeswap;
pub mod staking;
pub mod universal_router;
pub mod yo;

use chain_traits::transaction_parser::{ParseContext as GenericParseContext, parse_transaction};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::Num;

use super::model::{Transaction, TransactionReceipt};
use crate::ethereum_address_checksum;
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionType};

use self::{across::AcrossParser, mayan::MayanParser, okx::OkxParser, pancakeswap::PancakeSwapParser, universal_router::UniversalRouterParser, yo::YoParser};

pub use chain_traits::transaction_parser::TransactionParser;

pub const EVENT_WORD_SIZE: usize = 64;

pub struct ParseMetadata<'a> {
    pub chain: &'a Chain,
    pub receipt: &'a TransactionReceipt,
}

pub type ParseContext<'a> = GenericParseContext<'a, Transaction, ParseMetadata<'a>>;
pub type ProtocolParser = dyn for<'a> TransactionParser<ParseContext<'a>, PrimitivesTransaction>;

pub trait ParseContextExt {
    fn is_to(&self, address: &str) -> bool;
    fn is_to_any(&self, addresses: &[&str]) -> bool;
    fn make_swap_transaction(&self, from: &str, to: &str, metadata: &TransactionSwapMetadata) -> Option<PrimitivesTransaction>;
}

impl ParseContextExt for ParseContext<'_> {
    fn is_to(&self, address: &str) -> bool {
        self.is_to_any(&[address])
    }

    fn is_to_any(&self, addresses: &[&str]) -> bool {
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
            self.metadata.receipt.get_state(),
            self.metadata.receipt.get_fee(),
            AssetId::from_chain(*self.metadata.chain),
            self.transaction.value.clone(),
            None,
            serde_json::to_value(metadata).ok(),
            self.created_at,
        ))
    }
}

pub fn ethereum_value_from_log_data(data: &str, start: usize, end: usize) -> Option<BigUint> {
    data.trim_start_matches("0x").get(start..end).and_then(|s| BigUint::from_str_radix(s, 16).ok())
}

pub struct ProtocolParsers;

impl ProtocolParsers {
    fn default_parsers() -> [&'static ProtocolParser; 6] {
        [&AcrossParser, &MayanParser, &OkxParser, &YoParser, &PancakeSwapParser, &UniversalRouterParser]
    }

    pub fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt, created_at: DateTime<Utc>) -> Option<PrimitivesTransaction> {
        Self::map_transaction_with_parsers(chain, transaction, receipt, created_at, &[])
    }

    pub fn map_transaction_with_parsers(
        chain: &Chain,
        transaction: &Transaction,
        receipt: &TransactionReceipt,
        created_at: DateTime<Utc>,
        parsers: &[&'static ProtocolParser],
    ) -> Option<PrimitivesTransaction> {
        let context = ParseContext::new(transaction, created_at, ParseMetadata { chain, receipt });

        parse_transaction(&context, parsers.iter().copied().chain(Self::default_parsers()))
    }
}
