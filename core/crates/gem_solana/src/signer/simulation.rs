use super::{instructions, transaction};
use primitives::{SignerError, SignerInput, TransactionInputType, TransferDataOutputType, swap::SwapQuoteDataType};
use solana_primitives::{Pubkey, VersionedTransaction};

pub(crate) fn transaction_for_simulation(input: &SignerInput) -> Result<VersionedTransaction, SignerError> {
    let sender = || Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display);
    match &input.input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
            let sender = sender()?;
            let instructions = if asset.id.is_token() {
                instructions::token_transfer(input, sender)?
            } else {
                let mut estimation_input = input.clone();
                if input.is_max_value {
                    estimation_input.input.value = 0u8.into();
                }
                instructions::native_transfer(&estimation_input, sender)?
            };
            transaction::build_legacy_transaction(sender, transaction::block_hash(input)?, instructions)
        }
        TransactionInputType::TransferNft(_, _) => {
            let sender = sender()?;
            transaction::build_legacy_transaction(sender, transaction::block_hash(input)?, instructions::nft_transfer(input, sender)?)
        }
        TransactionInputType::Stake(_, _) => {
            let sender = sender()?;
            transaction::build_legacy_transaction(sender, transaction::block_hash(input)?, instructions::stake(input, sender)?)
        }
        TransactionInputType::Swap(from_asset, _, swap_data) => match swap_data.data.data_type {
            SwapQuoteDataType::Contract => transaction::prepare_with_fee(&swap_data.data.data, &input.fee),
            SwapQuoteDataType::Transfer => {
                let is_token = from_asset.id.is_token();
                let value = if input.is_max_value && !is_token {
                    input.value.clone()
                } else {
                    swap_data.quote.from_value.clone()
                };
                let mut rewritten = input.clone();
                rewritten.input.input_type = TransactionInputType::Transfer(from_asset.clone());
                rewritten.input.destination_address = swap_data.data.to.clone();
                rewritten.input.value = value;
                rewritten.input.memo = swap_data.data.memo.clone();
                transaction_for_simulation(&rewritten)
            }
        },
        TransactionInputType::Generic(_, _, extra) => {
            let data = extra.data_as_str().map_err(SignerError::invalid_input)?;
            match extra.output_type {
                TransferDataOutputType::EncodedTransaction => transaction::prepare_with_fee(data, &input.fee),
                TransferDataOutputType::Signature => crate::decode_transaction(data).map_err(SignerError::invalid_input),
            }
        }
        _ => SignerError::invalid_input_err("unsupported Solana transaction type"),
    }
}
