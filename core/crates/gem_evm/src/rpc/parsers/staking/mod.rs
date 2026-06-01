mod everstake;
mod monad;
mod smartchain;
mod transaction;

#[cfg(test)]
use chrono::DateTime;
#[cfg(test)]
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionState, TransactionType};

#[cfg(test)]
use crate::rpc::model::{Transaction, TransactionReciept, TransactionReplayTrace};

#[cfg(test)]
use super::ProtocolParsers;
use super::{ParseContext, ProtocolParser, ethereum_value_from_log_data};
use transaction::make_staking_transaction;

pub use everstake::EverstakeParser;
pub use monad::MonadStakingParser;
pub use smartchain::SmartChainStakingParser;

const EVENT_WORD_SIZE: usize = 64;

#[cfg(test)]
fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReciept, trace: Option<&TransactionReplayTrace>) -> PrimitivesTransaction {
    ProtocolParsers::map_transaction(chain, transaction, receipt, trace, None, DateTime::default()).unwrap()
}

#[cfg(test)]
fn assert_staking_transaction(transaction: &PrimitivesTransaction, chain: Chain, transaction_type: TransactionType, from: &str, to: &str, contract: &str, value: &str) {
    assert_eq!(transaction.transaction_type, transaction_type);
    assert_eq!(transaction.state, TransactionState::Confirmed);
    assert_eq!(transaction.asset_id, AssetId::from_chain(chain));
    assert_eq!(transaction.fee_asset_id, AssetId::from_chain(chain));
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.contract.as_deref(), Some(contract));
    assert_eq!(transaction.value, value);
    assert_eq!(transaction.metadata, None);
}
