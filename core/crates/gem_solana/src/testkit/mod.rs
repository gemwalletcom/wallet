mod account_data_mock;
mod block_transaction_mock;
mod epoch_info_mock;
mod instruction_mock;
mod lookup_table_mock;
mod message_mock;
mod pubkey_mock;
mod siws_mock;
mod token_account_info_mock;
mod token_balance_mock;
mod transaction_mock;

pub(crate) use pubkey_mock::test_wallet_pubkey;
pub(crate) use siws_mock::mock_siws_message;
#[cfg(feature = "signer")]
pub(crate) use transaction_mock::mock_legacy_transaction;
pub(crate) use transaction_mock::{mock_transaction, mock_transaction_with_accounts, mock_v0_transaction, mock_v1_transaction};

pub(crate) const TEST_BLOCKHASH: [u8; 32] = [1; 32];
