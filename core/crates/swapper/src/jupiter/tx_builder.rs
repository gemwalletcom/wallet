use std::iter::once;

use gem_encoding::encode_base64;
use gem_solana::{
    AddressLookupTableAccount, MAX_COMPUTE_UNIT_LIMIT, MAX_TRANSACTION_SIZE, Pubkey, TransactionBuilder,
    compute_budget::{parse_compute_unit_limit_data, set_compute_unit_limit, with_compute_unit_price},
};

use super::model::{BuildRequest, BuildResponse};
use super::swap_instructions::{QuotedRoute, RouteDestination, SwapInstructions};
use crate::{SwapperError, fees::subtract_bps};

impl BuildResponse {
    pub(super) fn lookup_table_addresses(&self) -> Vec<String> {
        self.addresses_by_lookup_table_address.iter().flatten().map(|(key, _)| key.to_string()).collect()
    }

    pub(super) fn into_transaction(self, request: &BuildRequest, lookup_tables: &[AddressLookupTableAccount]) -> Result<String, SwapperError> {
        if self.tip_instruction.is_some() || !self.other_instructions.is_empty() {
            return Err(SwapperError::InvalidRoute);
        }
        if !self.swap_instruction.accounts.iter().any(|account| account.pubkey == request.fee_account && account.is_writable) {
            return Err(SwapperError::compute_quote_error("Jupiter referral fee is missing from the swap instruction"));
        }

        let payer = Pubkey::from_base58(&request.taker).map_err(SwapperError::transaction_error)?;
        let quoted = QuotedRoute {
            wallet: payer,
            input_mint: Pubkey::from_base58(&request.input_mint).map_err(SwapperError::transaction_error)?,
            amount: request.amount.parse::<u64>().map_err(SwapperError::transaction_error)?,
            minimum_out: subtract_bps(&self.out_amount.parse::<u64>().map_err(SwapperError::compute_quote_error)?, request.slippage_bps),
            platform_fee_bps: u16::try_from(request.platform_fee_bps).map_err(SwapperError::compute_quote_error)?,
            destination: RouteDestination::WalletAccount(Pubkey::from_base58(&request.output_mint).map_err(SwapperError::transaction_error)?),
        };
        let swap = SwapInstructions::new(self.compute_budget_instructions, self.setup_instructions, self.swap_instruction, self.cleanup_instruction)?;
        swap.verify(&quoted)?;

        let compute_budget = with_compute_unit_price(
            once(set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT))
                .chain(swap.compute_budget.into_iter().filter(|instruction| parse_compute_unit_limit_data(&instruction.data).is_none()))
                .collect(),
            0,
        );
        let instructions = [compute_budget, swap.setup, vec![swap.swap], swap.cleanup.into_iter().collect()].concat();

        let transaction = TransactionBuilder::build_v0_transaction(payer, self.blockhash_with_metadata.blockhash, &instructions, lookup_tables).map_err(SwapperError::transaction_error)?;
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
    use super::*;
    use gem_solana::{USDC_TOKEN_MINT, decode_transaction};
    use primitives::SolanaAccountMeta;

    #[test]
    fn test_into_transaction() {
        let transaction = BuildResponse::mock().into_transaction(&BuildRequest::mock(), &BuildResponse::mock().mock_lookup_tables()).unwrap();
        let decoded = decode_transaction(&transaction).unwrap();
        assert_eq!(decoded.get_compute_unit_limit(), Some(MAX_COMPUTE_UNIT_LIMIT));
        assert_eq!(decoded.get_compute_unit_price(), Some(9054), "the reply's priority fee stays until the signer sets the wallet's own");
        assert!(BuildResponse::mock_usdc_sol().into_transaction(&BuildRequest::mock_usdc_sol(), &BuildResponse::mock_usdc_sol().mock_lookup_tables()).is_ok());

        assert_eq!(
            BuildResponse::mock().into_transaction(&BuildRequest::mock_usdc_sol(), &[]),
            Err(SwapperError::InvalidRoute),
            "a reply for another pair than the one quoted"
        );
        assert_eq!(
            BuildResponse {
                out_amount: "6900000".to_string(),
                ..BuildResponse::mock()
            }
            .into_transaction(&BuildRequest::mock(), &[]),
            Err(SwapperError::InvalidRoute),
            "a reply that shows more than its route delivers"
        );
        assert_eq!(
            BuildResponse::mock().into_transaction(&BuildRequest { slippage_bps: 50, ..BuildRequest::mock() }, &[]),
            Err(SwapperError::InvalidRoute),
            "a route that allows more slippage than the wallet asked for"
        );
        assert_eq!(
            BuildResponse::mock().into_transaction(
                &BuildRequest {
                    fee_account: USDC_TOKEN_MINT.to_string(),
                    ..BuildRequest::mock()
                },
                &[]
            ),
            Err(SwapperError::ComputeQuoteError("Jupiter referral fee is missing from the swap instruction".to_string()))
        );
        assert_eq!(
            BuildResponse {
                other_instructions: vec![BuildResponse::mock().swap_instruction],
                ..BuildResponse::mock()
            }
            .into_transaction(&BuildRequest::mock(), &[]),
            Err(SwapperError::InvalidRoute),
            "an instruction outside the swap's steps"
        );

        let mut build = BuildResponse::mock();
        build.swap_instruction.accounts.push(SolanaAccountMeta {
            pubkey: USDC_TOKEN_MINT.to_string(),
            is_signer: true,
            is_writable: false,
        });
        let lookup_tables = build.mock_lookup_tables();
        assert_eq!(
            build.into_transaction(&BuildRequest::mock(), &lookup_tables),
            Err(SwapperError::TransactionError("Jupiter transaction requires more than one signer".to_string()))
        );
    }
}
