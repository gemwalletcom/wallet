use super::{PROGRAM_ADDRESS, model::BuildResponse};
use crate::SwapperError;
use alloy_primitives::U256;
use gem_solana::{instruction_from_primitive, instructions_from_primitives};
use solana_primitives::{
    AddressLookupTableAccount, Pubkey, TransactionBuilder,
    compute_budget::{ensure_compute_unit_price, set_compute_unit_limit},
};

pub(super) const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;
pub(super) const MAX_TRANSACTION_SIZE: usize = 1_232;

impl BuildResponse {
    pub(super) fn validate(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: &str,
        taker: &str,
        slippage_bps: u32,
        platform_fee_bps: u32,
        fee_account: &str,
    ) -> Result<(), SwapperError> {
        if self.input_mint != input_mint || self.output_mint != output_mint || self.in_amount != amount || self.slippage_bps != slippage_bps || self.swap_mode != "ExactIn" {
            return Err(SwapperError::compute_quote_error("Jupiter build response does not match the request"));
        }
        if self.swap_instruction.program_id != PROGRAM_ADDRESS {
            return Err(SwapperError::compute_quote_error("Invalid Jupiter swap program"));
        }
        if !self.swap_instruction.accounts.iter().any(|account| account.pubkey == taker && account.is_signer) {
            return Err(SwapperError::compute_quote_error("Jupiter swap authority does not match the taker"));
        }
        if self.tip_instruction.is_some() {
            return Err(SwapperError::compute_quote_error("Unexpected Jupiter tip instruction"));
        }
        if platform_fee_bps > 0 && !self.swap_instruction.accounts.iter().any(|account| account.pubkey == fee_account && account.is_writable) {
            return Err(SwapperError::compute_quote_error("Jupiter referral fee is missing from the swap instruction"));
        }

        let out_amount = self.out_amount.parse::<U256>().map_err(SwapperError::compute_quote_error)?;
        let minimum_out_amount = self.other_amount_threshold.parse::<U256>().map_err(SwapperError::compute_quote_error)?;
        if minimum_out_amount > out_amount {
            return Err(SwapperError::compute_quote_error("Invalid Jupiter minimum output amount"));
        }
        Ok(())
    }

    pub(super) fn transaction_bytes(&self, payer: &str, compute_unit_limit: u32) -> Result<Vec<u8>, SwapperError> {
        let payer = Pubkey::from_base58(payer).map_err(SwapperError::transaction_error)?;

        let mut instructions = vec![set_compute_unit_limit(compute_unit_limit)];
        instructions.extend(instructions_from_primitives(self.compute_budget_instructions.clone()).map_err(SwapperError::transaction_error)?);
        ensure_compute_unit_price(&mut instructions, 0);
        instructions.extend(instructions_from_primitives(self.setup_instructions.clone()).map_err(SwapperError::transaction_error)?);
        instructions.push(instruction_from_primitive(self.swap_instruction.clone()).map_err(SwapperError::transaction_error)?);
        if let Some(cleanup_instruction) = self.cleanup_instruction.clone() {
            instructions.push(instruction_from_primitive(cleanup_instruction).map_err(SwapperError::transaction_error)?);
        }
        instructions.extend(instructions_from_primitives(self.other_instructions.clone()).map_err(SwapperError::transaction_error)?);

        let lookup_tables = self
            .addresses_by_lookup_table_address
            .as_ref()
            .into_iter()
            .flatten()
            .map(|(key, addresses)| AddressLookupTableAccount::new(*key, addresses.clone()))
            .collect::<Vec<_>>();

        let transaction =
            TransactionBuilder::build_v0_transaction(payer, self.blockhash_with_metadata.blockhash, &instructions, &lookup_tables).map_err(SwapperError::transaction_error)?;
        if transaction.num_required_signatures() != 1 {
            return Err(SwapperError::transaction_error("Jupiter transaction requires more than one signer"));
        }
        transaction.serialize().map_err(SwapperError::transaction_error)
    }
}

pub(super) fn buffered_compute_unit_limit(units_consumed: u64) -> Result<u32, SwapperError> {
    let buffered_units = units_consumed.saturating_mul(120).saturating_add(99) / 100;
    u32::try_from(buffered_units.min(u64::from(MAX_COMPUTE_UNIT_LIMIT))).map_err(SwapperError::transaction_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jupiter::model::BlockhashWithMetadata;
    use primitives::{SolanaAccountMeta, SolanaInstruction};
    use std::collections::BTreeMap;

    const PAYER: &str = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
    const INPUT_MINT: &str = gem_solana::WSOL_TOKEN_ADDRESS;
    const OUTPUT_MINT: &str = gem_solana::USDC_TOKEN_MINT;
    const FEE_ACCOUNT: &str = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";

    fn build_response() -> BuildResponse {
        BuildResponse {
            input_mint: INPUT_MINT.to_string(),
            output_mint: OUTPUT_MINT.to_string(),
            in_amount: "1000000000".to_string(),
            out_amount: "125000000".to_string(),
            other_amount_threshold: "123750000".to_string(),
            swap_mode: "ExactIn".to_string(),
            slippage_bps: 100,
            compute_budget_instructions: Vec::new(),
            setup_instructions: Vec::new(),
            swap_instruction: SolanaInstruction {
                program_id: PROGRAM_ADDRESS.to_string(),
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
    fn validates_and_builds_single_signer_transaction() {
        let build = build_response();

        build.validate(INPUT_MINT, OUTPUT_MINT, "1000000000", PAYER, 100, 50, FEE_ACCOUNT).unwrap();
        let transaction = build.transaction_bytes(PAYER, 420_000).unwrap();
        let decoded = gem_solana::decode_transaction(&gem_encoding::encode_base64(&transaction)).unwrap();

        assert!(transaction.len() <= MAX_TRANSACTION_SIZE);
        assert_eq!(decoded.get_compute_unit_limit(), Some(420_000));
        assert_eq!(decoded.get_compute_unit_price(), Some(0));
    }

    #[test]
    fn rejects_missing_referral_fee_account() {
        let error = build_response().validate(INPUT_MINT, OUTPUT_MINT, "1000000000", PAYER, 100, 50, OUTPUT_MINT).unwrap_err();

        assert!(error.to_string().contains("referral fee"));
    }

    #[test]
    fn rejects_additional_signer() {
        let mut build = build_response();
        build.swap_instruction.accounts.push(SolanaAccountMeta {
            pubkey: OUTPUT_MINT.to_string(),
            is_signer: true,
            is_writable: false,
        });

        assert!(build.transaction_bytes(PAYER, 420_000).unwrap_err().to_string().contains("more than one signer"));
    }

    #[test]
    fn buffers_and_caps_compute_units() {
        assert_eq!(buffered_compute_unit_limit(101).unwrap(), 122);
        assert_eq!(buffered_compute_unit_limit(u64::MAX).unwrap(), MAX_COMPUTE_UNIT_LIMIT);
    }
}
