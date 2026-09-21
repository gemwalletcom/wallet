use gem_encoding::encode_base64;
use gem_solana::{
    AddressLookupTableAccount, Base64InstructionData, DEFAULT_SWAP_GAS_LIMIT, JUPITER_PROGRAM_ID, MAX_TRANSACTION_SIZE, Pubkey, TransactionBuilder,
    compute_budget::{ensure_compute_unit_price, parse_compute_unit_limit_data, set_compute_unit_limit},
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

        let mut compute_budget_instructions = instructions_from_primitives::<Base64InstructionData>(self.compute_budget_instructions).map_err(SwapperError::transaction_error)?;
        compute_budget_instructions.retain(|instruction| parse_compute_unit_limit_data(&instruction.data).is_none());
        let mut instructions = vec![set_compute_unit_limit(DEFAULT_SWAP_GAS_LIMIT)];
        instructions.extend(compute_budget_instructions);
        ensure_compute_unit_price(&mut instructions, 0);
        instructions.extend(instructions_from_primitives::<Base64InstructionData>(self.setup_instructions).map_err(SwapperError::transaction_error)?);
        instructions.push(instruction_from_primitive::<Base64InstructionData>(self.swap_instruction).map_err(SwapperError::transaction_error)?);
        if let Some(cleanup_instruction) = self.cleanup_instruction {
            instructions.push(instruction_from_primitive::<Base64InstructionData>(cleanup_instruction).map_err(SwapperError::transaction_error)?);
        }
        instructions.extend(instructions_from_primitives::<Base64InstructionData>(self.other_instructions).map_err(SwapperError::transaction_error)?);

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
    use gem_solana::{DEFAULT_SWAP_GAS_LIMIT, USDC_TOKEN_MINT, decode_transaction};
    use primitives::SolanaAccountMeta;

    #[test]
    fn test_into_transaction() {
        let transaction = BuildResponse::mock().into_transaction(TEST_PAYER, TEST_FEE_ACCOUNT).unwrap();
        let decoded = decode_transaction(&transaction).unwrap();
        assert_eq!(decoded.get_compute_unit_limit(), Some(DEFAULT_SWAP_GAS_LIMIT));
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
