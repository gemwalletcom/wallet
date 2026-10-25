pub mod across;
pub mod aero;
pub mod mayan;
pub mod okx;
pub mod pancakeswap;
pub mod staking;
pub mod universal_router;
pub mod yo;

use chain_traits::transaction_parser::{ParseContext as GenericParseContext, parse_transaction};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::{CheckedSub, Num, Zero};

use super::{
    mapper::TRANSFER_TOPIC,
    model::{Log, Transaction, TransactionReceipt},
};
use crate::{address::ethereum_address_from_topic, ethereum_address_checksum};
use primitives::{
    AssetId, Chain, EVMChain, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType, contract_constants::EVM_NATIVE_TOKEN_ADDRESS, swap::EVM_REFERRAL_ADDRESS,
};

use self::{across::AcrossParser, aero::AeroParser, mayan::MayanParser, okx::OkxParser, pancakeswap::PancakeSwapParser, universal_router::UniversalRouterParser, yo::YoParser};

pub use chain_traits::transaction_parser::TransactionParser;

pub const EVENT_WORD_SIZE: usize = 64;
const WITHDRAWAL_TOPIC: &str = "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";

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

struct ReceiptTransfer {
    token: String,
    from: String,
    to: String,
    value: BigUint,
}

impl ReceiptTransfer {
    fn from_log(log: &Log) -> Option<Self> {
        if log.topics.len() != 3 || log.topics.first().is_none_or(|topic| topic != TRANSFER_TOPIC) {
            return None;
        }
        Some(Self {
            token: ethereum_address_checksum(&log.address).ok()?,
            from: ethereum_address_from_topic(log.topics.get(1)?)?,
            to: ethereum_address_from_topic(log.topics.get(2)?)?,
            value: ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?,
        })
    }
}

pub(super) fn transfer_swap(context: &ParseContext<'_>, provider: SwapProvider, native_fee: &BigUint) -> Option<TransactionSwapMetadata> {
    let chain = *context.metadata.chain;
    let wallet = ethereum_address_checksum(&context.transaction.from).ok()?;
    let transfers = context.metadata.receipt.logs.iter().filter_map(ReceiptTransfer::from_log).filter(|transfer| !transfer.value.is_zero()).collect::<Vec<_>>();
    let sent = transfers.iter().filter(|transfer| transfer.from == wallet).collect::<Vec<_>>();
    let received = transfers.iter().filter(|transfer| transfer.to == wallet).collect::<Vec<_>>();
    let ((from_asset, from_value), (to_asset, to_value)) = match (sent.as_slice(), received.as_slice()) {
        ([sent], [received]) if sent.token != received.token => ((AssetId::from_token(chain, &sent.token), sent.value.clone()), (AssetId::from_token(chain, &received.token), received.value.clone())),
        ([], [received]) if !context.transaction.value.is_zero() => ((AssetId::from_chain(chain), context.transaction.value.clone()), (AssetId::from_token(chain, &received.token), received.value.clone())),
        ([sent], []) => {
            let (weth, router) = (EVMChain::from_chain(chain)?.weth_contract()?, ethereum_address_checksum(context.transaction.to.as_deref()?).ok()?);
            let unwrapped = context.metadata.receipt.logs.iter().filter_map(|log| withdrawal(log, weth)).find(|(unwrapper, _)| *unwrapper == router)?.1;
            ((AssetId::from_token(chain, &sent.token), sent.value.clone()), (AssetId::from_chain(chain), unwrapped.checked_sub(native_fee)?))
        }
        _ => return None,
    };
    let (from_asset, from_value) = from_asset.mirror_to_native(from_value);
    let (to_asset, to_value) = to_asset.mirror_to_native(to_value);
    Some(TransactionSwapMetadata::new(from_asset, from_value, to_asset, to_value, provider))
}

pub(super) fn withdraw_value_from_receipt(token: &str, receipt: &TransactionReceipt) -> Option<BigUint> {
    receipt.logs.iter().find_map(|log| withdrawal(log, token)).map(|(_, value)| value)
}

fn withdrawal(log: &Log, token: &str) -> Option<(String, BigUint)> {
    if log.topics.len() != 2 || log.topics.first().is_none_or(|topic| topic != WITHDRAWAL_TOPIC) || !log.address.eq_ignore_ascii_case(token) {
        return None;
    }
    Some((ethereum_address_from_topic(log.topics.get(1)?)?, ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?))
}

pub fn ethereum_value_from_log_data(data: &str, start: usize, end: usize) -> Option<BigUint> {
    data.trim_start_matches("0x").get(start..end).and_then(|s| BigUint::from_str_radix(s, 16).ok())
}

pub struct ProtocolParsers;

impl ProtocolParsers {
    fn default_parsers() -> [&'static ProtocolParser; 7] {
        [&AcrossParser, &AeroParser, &MayanParser, &OkxParser, &YoParser, &PancakeSwapParser, &UniversalRouterParser]
    }

    pub fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt, created_at: DateTime<Utc>) -> Option<PrimitivesTransaction> {
        Self::map_transaction_with_parsers(chain, transaction, receipt, created_at, &[])
    }

    pub fn map_transaction_with_parsers(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt, created_at: DateTime<Utc>, parsers: &[&'static ProtocolParser]) -> Option<PrimitivesTransaction> {
        let context = ParseContext::new(transaction, created_at, ParseMetadata { chain, receipt });

        parse_transaction(&context, parsers.iter().copied().chain(Self::default_parsers()))
    }
}
