//! JSON emitted by Mysten's TypeScript transaction builder `Transaction.toJSON()`.

mod model;

#[cfg(feature = "rpc")]
mod builder;
#[cfg(feature = "rpc")]
mod finish;
#[cfg(feature = "rpc")]
mod replay;
#[cfg(feature = "rpc")]
mod resolver;

pub use model::*;

pub fn is_transaction_json(data: &[u8]) -> bool {
    data.trim_ascii_start().starts_with(b"{")
}

#[cfg(feature = "rpc")]
pub use finish::{finish_transaction_json, finish_transaction_json_from_sender};
#[cfg(feature = "rpc")]
pub use replay::{ReplayedTransaction, TransactionJsonReplay, prepare_transaction_json_replay};
