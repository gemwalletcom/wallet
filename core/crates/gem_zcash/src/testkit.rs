#[cfg(feature = "chain_integration_tests")]
use gem_bitcoin::BitcoinClient;
#[cfg(feature = "chain_integration_tests")]
use gem_reqwest::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use primitives::BitcoinChain;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(feature = "chain_integration_tests")]
use crate::ZcashProvider;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, DateTime, Transaction, TransactionState, TransactionType, transaction_utxo::TransactionUtxoInput};

pub(crate) const TEST_SHIELDED_RECEIVE_ID: &str = "13ab80b092baae27d3c053bff2367d06e8a9e61e99f13eb907bd8c92825c78b8";
pub(crate) const TEST_RECEIVER_ADDRESS: &str = "t1SXEuGoJQco68tbdW6guj5w2CjGunQD347";
pub(crate) const TEST_TRANSPARENT_TRANSFER_ID: &str = "592dfdfa7d053f06defb813cf5f65cd87fc64eba1ae1973b3273d8a836b37866";
pub(crate) const TEST_TRANSPARENT_SENDER_ADDRESS: &str = "t1JKiq1Zvi3ZcR25eSHekhantaPUK9dT6yc";
pub(crate) const TEST_TRANSPARENT_RECIPIENT_ADDRESS: &str = "t1YQV51DKzKP63xJcynXuRfryMjfmgTJ7Jc";

const TEST_BLOCK_3500000_TIME: i64 = 1_790_662_825;

pub(crate) fn mock_transparent_transfer() -> Transaction {
    mock_zcash_transfer(
        TEST_TRANSPARENT_TRANSFER_ID,
        10_000,
        80_000,
        TransactionUtxoInput {
            address: TEST_TRANSPARENT_SENDER_ADDRESS.to_string(),
            value: BigUint::from(90_000u64),
        },
        TransactionUtxoInput {
            address: TEST_TRANSPARENT_RECIPIENT_ADDRESS.to_string(),
            value: BigUint::from(80_000u64),
        },
    )
}

pub(crate) fn mock_shielded_receive() -> Transaction {
    mock_zcash_transfer(
        TEST_SHIELDED_RECEIVE_ID,
        15_000,
        2_000_000,
        TransactionUtxoInput {
            address: String::new(),
            value: BigUint::from(2_015_000u64),
        },
        TransactionUtxoInput {
            address: TEST_RECEIVER_ADDRESS.to_string(),
            value: BigUint::from(2_000_000u64),
        },
    )
}

fn mock_zcash_transfer(hash: &str, fee: u64, value: u64, input: TransactionUtxoInput, output: TransactionUtxoInput) -> Transaction {
    Transaction::new_with_utxo(
        hash.to_string(),
        AssetId::from_chain(Chain::Zcash),
        TransactionType::Transfer,
        TransactionState::Confirmed,
        BigUint::from(fee),
        AssetId::from_chain(Chain::Zcash),
        BigUint::from(value),
        None,
        Some(vec![input]),
        Some(vec![output]),
        None,
        DateTime::from_timestamp(TEST_BLOCK_3500000_TIME),
    )
}

#[cfg(feature = "chain_integration_tests")]
pub(crate) const TEST_ZCASH_WALLET_ADDRESS: &str = "t1dTLYuecTCt8irWJsyLkDaJm2EAvVgdxgM";

#[cfg(feature = "chain_integration_tests")]
pub(crate) fn create_zcash_test_provider() -> ZcashProvider<ReqwestClient> {
    let settings = get_test_settings();
    ZcashProvider::new(BitcoinClient::new(ReqwestClient::new_test_client(settings.chains.zcash.url), BitcoinChain::Zcash))
}
