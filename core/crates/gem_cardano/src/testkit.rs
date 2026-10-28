use primitives::{OutputMinimum, TransactionLoadMetadata, UTXO};

use crate::{
    address::ShelleyAddress,
    planner::MIN_OUTPUT_LOVELACE,
    transaction::{TransactionInput, TransactionOutput},
};

pub(crate) const TEST_BLOCK_NUMBER: u64 = 189_992_800;

pub(crate) fn mock_cardano_metadata(utxos: Vec<UTXO>) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Cardano {
        utxos,
        block_number: TEST_BLOCK_NUMBER,
        output_minimum: OutputMinimum {
            payment: MIN_OUTPUT_LOVELACE,
            change: MIN_OUTPUT_LOVELACE,
        },
    }
}

impl TransactionInput {
    pub(crate) fn mock() -> Self {
        Self::mock_with("f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767", 1)
    }

    pub(crate) fn mock_with(transaction_hash: &str, output_index: u64) -> Self {
        Self {
            transaction_hash: hex::decode(transaction_hash).unwrap().try_into().unwrap(),
            output_index,
        }
    }
}

impl TransactionOutput {
    pub(crate) fn mock() -> Self {
        Self::mock_with("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23", 2_000_000)
    }

    pub(crate) fn mock_with(address: &str, amount: u64) -> Self {
        Self {
            address: ShelleyAddress::parse(address).unwrap().as_bytes().to_vec(),
            amount,
        }
    }
}
