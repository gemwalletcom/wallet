use crate::{
    address::ShelleyAddress,
    transaction::{TransactionInput, TransactionOutput},
};

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
        Self::mock_with(
            "addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23",
            2_000_000,
        )
    }

    pub(crate) fn mock_with(address: &str, amount: u64) -> Self {
        Self {
            address: ShelleyAddress::parse(address).unwrap().as_bytes().to_vec(),
            amount,
        }
    }
}
