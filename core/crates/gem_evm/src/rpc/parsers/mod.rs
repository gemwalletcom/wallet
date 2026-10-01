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

use super::{
    mapper::TRANSFER_TOPIC,
    model::{Transaction, TransactionReceipt},
};
use crate::{address::ethereum_address_from_topic, ethereum_address_checksum};
use primitives::{AssetId, Chain, EVMChain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType, contract_constants::EVM_NATIVE_TOKEN_ADDRESS, swap::EVM_REFERRAL_ADDRESS};

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
        self.transaction.to.as_ref().is_some_and(|to| addresses.iter().any(|address| to.eq_ignore_ascii_case(address)))
    }

    fn make_swap_transaction(&self, from: &str, to: &str, metadata: &TransactionSwapMetadata) -> Option<PrimitivesTransaction> {
        let from = ethereum_address_checksum(from).ok()?;
        let to = ethereum_address_checksum(to).ok()?;
        let contract = self.transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());
        let metadata = match from == EVM_REFERRAL_ADDRESS {
            true => metadata.clone().with_referral_fee(None),
            false => metadata.clone(),
        };

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

pub(super) fn referral_fee_from_transfers(chain: Chain, receipt: &TransactionReceipt) -> Option<TransactionSwapReferralFee> {
    receipt.logs.iter().find_map(|log| {
        let is_referral_transfer = log.topics.len() == 3 && log.topics.first()? == TRANSFER_TOPIC && ethereum_address_from_topic(log.topics.get(2)?)? == EVM_REFERRAL_ADDRESS;
        if !is_referral_transfer {
            return None;
        }
        referral_fee_for_token(chain, &log.address, ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?)
    })
}

pub(super) fn referral_fee_for_token(chain: Chain, token: &str, value: BigUint) -> Option<TransactionSwapReferralFee> {
    let token = ethereum_address_checksum(token).ok()?;
    let is_native = token == EVM_NATIVE_TOKEN_ADDRESS || EVMChain::from_chain(chain).and_then(|chain| chain.weth_contract()).is_some_and(|contract| contract.eq_ignore_ascii_case(&token));
    let (asset_id, value) = match is_native {
        true => (AssetId::from_chain(chain), value),
        false => AssetId::from_token(chain, &token).mirror_to_native(value),
    };
    Some(TransactionSwapReferralFee { asset_id, value })
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

    pub fn map_transaction_with_parsers(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt, created_at: DateTime<Utc>, parsers: &[&'static ProtocolParser]) -> Option<PrimitivesTransaction> {
        let context = ParseContext::new(transaction, created_at, ParseMetadata { chain, receipt });

        parse_transaction(&context, parsers.iter().copied().chain(Self::default_parsers()))
    }
}
