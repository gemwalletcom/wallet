use super::model::BuildResponse;
use crate::SwapperError;
use gem_encoding::encode_base64;
use gem_solana::{DEFAULT_SWAP_GAS_LIMIT, JUPITER_PROGRAM_ID, instruction_from_primitive, instructions_from_primitives};
use solana_primitives::{
    AddressLookupTableAccount, MAX_TRANSACTION_SIZE, Pubkey, TransactionBuilder,
    compute_budget::{ensure_compute_unit_price, parse_compute_unit_limit_data, set_compute_unit_limit},
};

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

        let mut compute_budget_instructions = instructions_from_primitives(self.compute_budget_instructions).map_err(SwapperError::transaction_error)?;
        compute_budget_instructions.retain(|instruction| parse_compute_unit_limit_data(&instruction.data).is_none());
        let mut instructions = vec![set_compute_unit_limit(DEFAULT_SWAP_GAS_LIMIT)];
        instructions.extend(compute_budget_instructions);
        ensure_compute_unit_price(&mut instructions, 0);
        instructions.extend(instructions_from_primitives(self.setup_instructions).map_err(SwapperError::transaction_error)?);
        instructions.push(instruction_from_primitive(self.swap_instruction).map_err(SwapperError::transaction_error)?);
        if let Some(cleanup_instruction) = self.cleanup_instruction {
            instructions.push(instruction_from_primitive(cleanup_instruction).map_err(SwapperError::transaction_error)?);
        }
        instructions.extend(instructions_from_primitives(self.other_instructions).map_err(SwapperError::transaction_error)?);

        let lookup_tables = self
            .addresses_by_lookup_table_address
            .into_iter()
            .flatten()
            .map(|(key, addresses)| AddressLookupTableAccount::new(key, addresses))
            .collect::<Vec<_>>();

        let transaction =
            TransactionBuilder::build_v0_transaction(payer, self.blockhash_with_metadata.blockhash, &instructions, &lookup_tables).map_err(SwapperError::transaction_error)?;
        if transaction.num_required_signatures() != 1 {
            return Err(SwapperError::transaction_error("Jupiter transaction requires more than one signer"));
        }
        let transaction = transaction.serialize().map_err(SwapperError::transaction_error)?;
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(SwapperError::transaction_error(format!(
                "Jupiter transaction size {} exceeds maximum of {} bytes",
                transaction.len(),
                MAX_TRANSACTION_SIZE
            )));
        }
        Ok(encode_base64(&transaction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jupiter::model::BlockhashWithMetadata;
    use gem_solana::{DEFAULT_SWAP_GAS_LIMIT, USDC_TOKEN_MINT, decode_transaction};
    use primitives::{SolanaAccountMeta, SolanaInstruction};
    use std::collections::BTreeMap;

    const PAYER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
    const FEE_ACCOUNT: &str = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";

    fn build_response() -> BuildResponse {
        BuildResponse {
            out_amount: "125000000".to_string(),
            slippage_bps: 100,
            compute_budget_instructions: Vec::new(),
            setup_instructions: Vec::new(),
            swap_instruction: SolanaInstruction {
                program_id: JUPITER_PROGRAM_ID.to_string(),
                accounts: vec![
                    SolanaAccountMeta {
                        pubkey: PAYER.to_string(),
                        is_signer: true,
                        is_writable: true,
                    },
                    SolanaAccountMeta {
                        pubkey: FEE_ACCOUNT.to_string(),
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: String::new(),
            },
            cleanup_instruction: None,
            other_instructions: Vec::new(),
            tip_instruction: None,
            addresses_by_lookup_table_address: Some(BTreeMap::new()),
            blockhash_with_metadata: BlockhashWithMetadata { blockhash: [0; 32] },
        }
    }

    #[test]
    fn test_into_transaction() {
        let transaction = build_response().into_transaction(PAYER, FEE_ACCOUNT).unwrap();
        let decoded = decode_transaction(&transaction).unwrap();
        assert_eq!(decoded.get_compute_unit_limit(), Some(DEFAULT_SWAP_GAS_LIMIT));
        assert_eq!(decoded.get_compute_unit_price(), Some(0));

        assert_eq!(
            build_response().into_transaction(PAYER, USDC_TOKEN_MINT),
            Err(SwapperError::ComputeQuoteError("Jupiter referral fee is missing from the swap instruction".to_string()))
        );

        let mut build = build_response();
        build.swap_instruction.accounts.push(SolanaAccountMeta {
            pubkey: USDC_TOKEN_MINT.to_string(),
            is_signer: true,
            is_writable: false,
        });
        assert_eq!(
            build.into_transaction(PAYER, FEE_ACCOUNT),
            Err(SwapperError::TransactionError("Jupiter transaction requires more than one signer".to_string()))
        );
    }
}
