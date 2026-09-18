use std::fmt::Display;

use crate::SolanaError;

mod account;
mod instruction;
mod message;
mod pda;
mod pubkey;
mod signature;
mod transaction;

pub use account::{AddressLookupTableAccount, MessageAddressTableLookup};
pub use instruction::{AccountMeta, CompiledInstruction, Instruction};
pub use message::{Message, MessageHeader, TransactionConfig, VersionedMessageV0, VersionedMessageV1};
pub use pda::find_program_address;
pub use pubkey::Pubkey;
pub use signature::SignatureBytes;
pub use transaction::VersionedTransaction;

#[cfg(feature = "signer")]
pub(crate) use message::{MESSAGE_VERSION_PREFIX, OFFCHAIN_MESSAGE_PREFIX};

pub const MAX_TRANSACTION_SIZE: usize = 1232;
pub(crate) const MAX_V1_TRANSACTION_SIZE: usize = 4096;
pub(crate) const MAX_ACCOUNT_KEYS: usize = u8::MAX as usize + 1;

pub(crate) fn invalid_transaction(reason: impl Display) -> SolanaError {
    SolanaError::invalid_input(format!("Invalid Solana transaction: {reason}"))
}
