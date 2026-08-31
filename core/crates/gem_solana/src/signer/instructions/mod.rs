mod nft_transfer;
mod stake;
mod stake_account;
mod token_transfer;
mod transfer;

use primitives::{SignerError, SignerInput};
use solana_primitives::{AccountMeta, Pubkey};

pub(super) use nft_transfer::nft_transfer;
pub(super) use stake::stake;
pub(super) use token_transfer::token_transfer;
pub(super) use transfer::native_transfer;

fn reference_accounts(input: &SignerInput) -> Result<Vec<AccountMeta>, SignerError> {
    input
        .metadata
        .get_solana_references()?
        .iter()
        .map(|reference| Pubkey::from_base58(reference).map(AccountMeta::new_readonly).map_err(SignerError::from_display))
        .collect()
}
