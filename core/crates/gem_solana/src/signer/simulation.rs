use super::transaction;
use crate::decode_transaction;
use primitives::{SignerError, SignerInput, TransactionInputType, TransferDataOutputType, swap::SwapQuoteDataType};
use solana_primitives::VersionedTransaction;

pub(crate) fn transaction_for_simulation(input: &SignerInput) -> Result<VersionedTransaction, SignerError> {
    match &input.input.input_type {
        TransactionInputType::Swap(_, _, swap_data) => match swap_data.data.data_type {
            SwapQuoteDataType::Contract => transaction::prepare_with_fee(&swap_data.data.data, &input.fee),
            SwapQuoteDataType::Transfer => SignerError::invalid_input_err("unsupported Solana transaction type"),
        },
        TransactionInputType::Generic(_, _, extra) => {
            let data = extra.data_as_str().map_err(SignerError::invalid_input)?;
            match extra.output_type {
                TransferDataOutputType::EncodedTransaction => transaction::prepare_with_fee(data, &input.fee),
                TransferDataOutputType::Signature => decode_transaction(data).map_err(SignerError::invalid_input),
            }
        }
        _ => SignerError::invalid_input_err("unsupported Solana transaction type"),
    }
}
