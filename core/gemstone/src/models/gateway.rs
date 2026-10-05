use chain_primitives::checksum_address;

use primitives::{BroadcastOptions, TransactionInputType, TransactionPreloadInput};

pub type GemBroadcastOptions = BroadcastOptions;

#[derive(Debug, Clone)]
pub struct GemTransactionPreloadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub references: Vec<String>,
}

impl From<TransactionPreloadInput> for GemTransactionPreloadInput {
    fn from(input: TransactionPreloadInput) -> Self {
        Self {
            input_type: input.input_type,
            sender_address: input.sender_address,
            destination_address: input.destination_address,
            references: input.references,
        }
    }
}

impl From<GemTransactionPreloadInput> for TransactionPreloadInput {
    fn from(input: GemTransactionPreloadInput) -> Self {
        let input_type: TransactionInputType = input.input_type;
        let destination_address = checksum_address(&input.destination_address, input_type.get_asset().chain());
        Self {
            input_type,
            sender_address: input.sender_address,
            destination_address,
            references: input.references,
        }
    }
}
