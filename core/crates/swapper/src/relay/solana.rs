use std::sync::Arc;

use gem_solana::{HexInstructionData, Pubkey, StandardInstruction, decode_instructions};
use primitives::Chain;

use super::{contracts::contracts, model::SolanaStepData};
use crate::{SwapperError, SwapperQuoteData, alien::RpcProvider, contract_check::solana::verify_steps, solana_dex::SolanaTransaction};

pub async fn build_quote_data(wallet_address: &str, step: &SolanaStepData, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
    transaction(step, wallet_address)?.quote_data(wallet_address, rpc_provider).await
}

fn transaction(step: &SolanaStepData, wallet_address: &str) -> Result<SolanaTransaction, SwapperError> {
    let instructions = decode_instructions::<HexInstructionData>(step.instructions.clone()).map_err(SwapperError::transaction_error)?;
    let steps: Vec<StandardInstruction> = instructions.iter().map(StandardInstruction::new).collect();
    verify_steps(&steps, &Pubkey::from_base58(wallet_address).map_err(SwapperError::transaction_error)?, &contracts(Chain::Solana).targets)?;
    Ok(SolanaTransaction::new(instructions, step.address_lookup_table_addresses.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        contract_check::solana::verify_solana,
        relay::{model::RelayQuoteResponse, testkit::TEST_SOLANA_PAYER},
    };
    use gem_solana::{AddressLookupTableAccount, decode_transaction};
    use num_bigint::BigUint;
    use primitives::{contract_constants::SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, hex, testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS};

    #[test]
    fn test_build_transaction() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_sol_to_base_usdc.json")).unwrap();
        let step = response.get_solana_step().unwrap();

        let quote_data = transaction(step, "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T").unwrap().encode("4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T", &[]).unwrap();
        verify_solana(&quote_data.data, "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T", &contracts(Chain::Solana).targets).unwrap();

        let transaction = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.get_compute_unit_price(), Some(0));
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert!(quote_data.approval.is_none());

        let instructions = decode_instructions::<HexInstructionData>(step.instructions.clone()).unwrap();
        assert_eq!(instructions[0].program_id.to_base58(), SOLANA_RELAY_DEPOSITORY_PROGRAM_ID);
        assert_eq!(instructions[0].data, hex::decode("0d9e0ddf5fd51c0600e1f50500000000d8f6831d9a771a5b031cc86256987f54a9759e6107db24b07ec66eef4e555055").unwrap());
    }

    #[test]
    fn test_build_transaction_through_jupiter() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_cbbtc_to_base_usdc.json")).unwrap();
        let step = response.get_solana_step().unwrap();
        let mut accounts = Vec::new();
        for account in decode_instructions::<HexInstructionData>(step.instructions.clone()).unwrap().into_iter().flat_map(|instruction| instruction.accounts) {
            if !account.is_signer && !accounts.contains(&account.pubkey) {
                accounts.push(account.pubkey);
            }
        }
        let lookup_table = AddressLookupTableAccount::new(Pubkey::new([7; 32]), accounts);

        let quote_data = transaction(step, TEST_SOLANA_PAYER).unwrap().encode(TEST_SOLANA_PAYER, &[lookup_table]).unwrap();

        verify_solana(&quote_data.data, TEST_SOLANA_PAYER, &contracts(Chain::Solana).targets).unwrap();

        let mut other_token_program = step.clone();
        other_token_program.instructions[0].accounts[5].pubkey = TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string();
        assert_eq!(
            transaction(&other_token_program, TEST_SOLANA_PAYER).err(),
            Some(SwapperError::InvalidRoute),
            "the steps are checked before a lookup table can hide an account"
        );
    }
}
