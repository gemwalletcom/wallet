use gem_encoding::encode_base64;
use gem_solana::{
    AddressLookupTableAccount, Base64InstructionData, JUPITER_PROGRAM_ID, MAX_COMPUTE_UNIT_LIMIT, MAX_TRANSACTION_SIZE, Pubkey, TransactionBuilder,
    compute_budget::{parse_compute_unit_limit_data, set_compute_unit_limit, with_compute_unit_price},
    instruction_from_primitive, instructions_from_primitives,
};

use super::model::BuildResponse;
use crate::SwapperError;

impl BuildResponse {
    pub(super) fn into_transaction(self, payer: &str, fee_account: &str) -> Result<String, SwapperError> {
        if self.swap_instruction.program_id != JUPITER_PROGRAM_ID {
            return Err(SwapperError::compute_quote_error("Invalid Jupiter swap program"));
        }
        if !self.swap_instruction.accounts.iter().any(|account| account.pubkey == payer && account.is_signer) {
            return Err(SwapperError::compute_quote_error("Jupiter swap authority does not match the taker"));
        }
        if self.tip_instruction.is_some() {
            return Err(SwapperError::compute_quote_error("Unexpected Jupiter tip instruction"));
        }
        if !self.swap_instruction.accounts.iter().any(|account| account.pubkey == fee_account && account.is_writable) {
            return Err(SwapperError::compute_quote_error("Jupiter referral fee is missing from the swap instruction"));
        }

        let payer = Pubkey::from_base58(payer).map_err(SwapperError::transaction_error)?;

        let compute_budget_instructions = instructions_from_primitives::<Base64InstructionData>(self.compute_budget_instructions)
            .map_err(SwapperError::transaction_error)?
            .into_iter()
            .filter(|instruction| parse_compute_unit_limit_data(&instruction.data).is_none());
        let compute_budget = with_compute_unit_price(std::iter::once(set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT)).chain(compute_budget_instructions).collect(), 0);
        let setup = instructions_from_primitives::<Base64InstructionData>(self.setup_instructions).map_err(SwapperError::transaction_error)?;
        let swap = instruction_from_primitive::<Base64InstructionData>(self.swap_instruction).map_err(SwapperError::transaction_error)?;
        let cleanup = self.cleanup_instruction.map(instruction_from_primitive::<Base64InstructionData>).transpose().map_err(SwapperError::transaction_error)?;
        let other = instructions_from_primitives::<Base64InstructionData>(self.other_instructions).map_err(SwapperError::transaction_error)?;
        let instructions = [compute_budget, setup, vec![swap], cleanup.into_iter().collect(), other].concat();

        let lookup_tables = self
            .addresses_by_lookup_table_address
            .into_iter()
            .flatten()
            .map(|(key, addresses)| AddressLookupTableAccount::new(key, addresses))
            .collect::<Vec<_>>();

        let transaction = TransactionBuilder::build_v0_transaction(payer, self.blockhash_with_metadata.blockhash, &instructions, &lookup_tables).map_err(SwapperError::transaction_error)?;
        if transaction.num_required_signatures() != 1 {
            return Err(SwapperError::transaction_error("Jupiter transaction requires more than one signer"));
        }
        let transaction = transaction.serialize().map_err(SwapperError::transaction_error)?;
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(SwapperError::transaction_error(format!("Jupiter transaction size {} exceeds maximum of {} bytes", transaction.len(), MAX_TRANSACTION_SIZE)));
        }
        Ok(encode_base64(&transaction))
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{TEST_FEE_ACCOUNT, TEST_PAYER};
    use super::*;
    use gem_solana::models::SolanaAccountMeta;
    use gem_solana::{USDC_TOKEN_MINT, decode_transaction};

    #[test]
    fn test_into_transaction() {
        let transaction = BuildResponse::mock().into_transaction(TEST_PAYER, TEST_FEE_ACCOUNT).unwrap();
        let decoded = decode_transaction(&transaction).unwrap();
        assert_eq!(decoded.get_compute_unit_limit(), Some(MAX_COMPUTE_UNIT_LIMIT));
        assert_eq!(decoded.get_compute_unit_price(), Some(0));

        assert_eq!(
            BuildResponse::mock().into_transaction(TEST_PAYER, USDC_TOKEN_MINT),
            Err(SwapperError::ComputeQuoteError("Jupiter referral fee is missing from the swap instruction".to_string()))
        );

        let mut build = BuildResponse::mock();
        build.swap_instruction.accounts.push(SolanaAccountMeta {
            pubkey: USDC_TOKEN_MINT.to_string(),
            is_signer: true,
            is_writable: false,
        });
        assert_eq!(
            build.into_transaction(TEST_PAYER, TEST_FEE_ACCOUNT),
            Err(SwapperError::TransactionError("Jupiter transaction requires more than one signer".to_string()))
        );
    }
}
