use gem_solana::{
    AddressLookupTableAccount, BLOCKHASH_SET_AT_SIGNING, Instruction, Pubkey, SolanaAddress, SolanaClient,
    associated_token::create_associated_token_account_idempotent,
    compute_budget::{self, set_compute_unit_limit, set_compute_unit_price},
    encode_v0_transaction, system,
    token::transfer_checked_with_program_id,
    wrapped_sol,
};
use num_bigint::BigUint;
use primitives::Chain;
use std::sync::Arc;

use super::{protocol::PoolSwap, token::SwapToken};
use crate::{
    SwapperError, SwapperQuoteData,
    alien::RpcProvider,
    client_factory::create_client_with_chain,
    fees::{FeeSide, ReferralTransfer},
};

#[derive(Debug)]
pub(crate) struct SolanaTransaction {
    instructions: Vec<Instruction>,
    lookup_table_addresses: Vec<String>,
}

impl SolanaTransaction {
    pub(crate) fn new(instructions: Vec<Instruction>, lookup_table_addresses: Vec<String>) -> Self {
        Self { instructions, lookup_table_addresses }
    }

    pub(crate) async fn quote_data(self, wallet_address: &str, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError> {
        let client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana)?);
        let lookup_tables = client.get_address_lookup_tables(self.lookup_table_addresses.clone()).await.map_err(SwapperError::transaction_error)?;
        self.encode(wallet_address, &lookup_tables)
    }

    pub(crate) fn encode(self, wallet_address: &str, lookup_tables: &[AddressLookupTableAccount]) -> Result<SwapperQuoteData, SwapperError> {
        let fee_payer = SolanaAddress::parse(wallet_address).map_err(SwapperError::transaction_error)?.into();
        let instructions = compute_budget::with_compute_unit_price(self.instructions, 0);
        let data = encode_v0_transaction(fee_payer, BLOCKHASH_SET_AT_SIGNING, &instructions, lookup_tables).map_err(SwapperError::transaction_error)?;
        let gas_limit = compute_budget::get_compute_unit_limit(&instructions).map(|limit| limit.to_string());
        Ok(SwapperQuoteData::new_contract(String::new(), BigUint::ZERO, data, None, gas_limit))
    }
}

#[derive(Debug)]
pub(crate) struct DexTransaction<'a> {
    pub swap: PoolSwap<'a>,
    pub referral: Pubkey,
    pub fee: ReferralTransfer,
    pub swap_instructions: Vec<Instruction>,
    pub intermediate_token_account: bool,
    pub compute_unit_limit: u32,
}

impl DexTransaction<'_> {
    pub fn encode(&self) -> Result<String, SwapperError> {
        encode_v0_transaction(self.swap.authority, BLOCKHASH_SET_AT_SIGNING, &self.instructions()?, &[]).map_err(SwapperError::transaction_error)
    }

    fn instructions(&self) -> Result<Vec<Instruction>, SwapperError> {
        let PoolSwap {
            authority,
            input,
            intermediate,
            output,
            amount,
            ..
        } = &self.swap;
        let intermediate = (*intermediate).filter(|_| self.intermediate_token_account);
        let input_fee = self.fee_instructions(FeeSide::Input, input)?;
        let wrap = if input.is_native {
            wrapped_sol::wrap_instructions(authority, *amount).map_err(SwapperError::transaction_error)?
        } else {
            vec![]
        };
        let token_accounts = intermediate
            .into_iter()
            .chain([*output])
            .map(|token| create_associated_token_account_idempotent(authority, authority, &token.mint, &token.token_program).map_err(SwapperError::transaction_error))
            .collect::<Result<Vec<_>, _>>()?;
        let unwraps_intermediate = intermediate.is_some_and(|token| token.mint == wrapped_sol::wrapped_sol_mint());
        let unwrap = if input.is_native || output.is_native || unwraps_intermediate {
            vec![wrapped_sol::unwrap_instruction(authority).map_err(SwapperError::transaction_error)?]
        } else {
            vec![]
        };
        let output_fee = self.fee_instructions(FeeSide::Output, output)?;
        Ok([
            vec![set_compute_unit_price(0), set_compute_unit_limit(self.compute_unit_limit)],
            input_fee,
            wrap,
            token_accounts,
            self.swap_instructions.clone(),
            unwrap,
            output_fee,
        ]
        .concat())
    }

    fn fee_instructions(&self, side: FeeSide, token: &SwapToken) -> Result<Vec<Instruction>, SwapperError> {
        if self.fee.side != side || self.fee.value == 0 {
            return Ok(vec![]);
        }
        let authority = &self.swap.authority;
        if token.is_native {
            return Ok(vec![system::transfer(authority, &self.referral, self.fee.value)]);
        }
        Ok(vec![transfer_checked_with_program_id(
            &token.account(authority)?,
            &token.mint,
            &token.account(&self.referral)?,
            authority,
            self.fee.value,
            token.decimals,
            &token.token_program,
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TEST_SOLANA_PAYER;
    use gem_solana::{
        DEFAULT_SWAP_GAS_LIMIT, decode_transaction,
        instructions::program_ids::{associated_token_program, compute_budget_program, system_program, token_program},
        wrapped_sol::wrapped_sol_mint,
    };
    use primitives::swap::SOLANA_REFERRAL_ADDRESS;

    fn program_ids(transaction: &DexTransaction) -> Vec<Pubkey> {
        transaction.instructions().unwrap().iter().map(|instruction| instruction.program_id).collect()
    }

    #[test]
    fn test_instructions() {
        let native = SwapToken {
            mint: wrapped_sol_mint(),
            is_native: true,
            ..SwapToken::mock()
        };
        let usdc = SwapToken::mock();
        let budget = [compute_budget_program(), compute_budget_program()];
        let swap = Pubkey::new([9; 32]);
        let input_fee = DexTransaction {
            fee: ReferralTransfer { side: FeeSide::Input, value: 5_000 },
            ..DexTransaction::mock(&native, &usdc)
        };

        assert_eq!(
            program_ids(&input_fee),
            [
                budget.as_slice(),
                &[system_program(), associated_token_program(), system_program(), token_program(), associated_token_program(), swap, token_program()]
            ]
            .concat()
        );
        assert_eq!(
            input_fee.instructions().unwrap()[2],
            system::transfer(&Pubkey::from_base58(TEST_SOLANA_PAYER).unwrap(), &Pubkey::from_base58(SOLANA_REFERRAL_ADDRESS).unwrap(), 5_000)
        );
        assert_eq!(
            program_ids(&DexTransaction::mock(&usdc, &native)),
            [budget.as_slice(), &[associated_token_program(), swap, token_program(), system_program()]].concat()
        );
        assert_eq!(
            program_ids(&DexTransaction {
                fee: ReferralTransfer { side: FeeSide::Output, value: 0 },
                ..DexTransaction::mock(&usdc, &native)
            }),
            [budget.as_slice(), &[associated_token_program(), swap, token_program()]].concat()
        );
    }

    #[test]
    fn test_intermediate_token_account() {
        let usdc = SwapToken::mock();
        let output = SwapToken {
            mint: Pubkey::new([7; 32]),
            ..SwapToken::mock()
        };
        let native = SwapToken {
            mint: wrapped_sol_mint(),
            ..SwapToken::mock()
        };
        let routed = |intermediate_token_account| {
            let transaction = DexTransaction::mock(&usdc, &output);
            DexTransaction {
                swap: PoolSwap {
                    intermediate: Some(&native),
                    ..transaction.swap
                },
                intermediate_token_account,
                ..transaction
            }
        };
        let budget = [compute_budget_program(), compute_budget_program()];
        let swap = Pubkey::new([9; 32]);

        assert_eq!(
            program_ids(&routed(true)),
            [budget.as_slice(), &[associated_token_program(), associated_token_program(), swap, token_program(), token_program()]].concat()
        );
        assert_eq!(program_ids(&routed(false)), [budget.as_slice(), &[associated_token_program(), swap, token_program()]].concat());
    }

    #[test]
    fn test_encode() {
        let usdc = SwapToken::mock();
        let decoded = decode_transaction(&DexTransaction::mock(&usdc, &usdc).encode().unwrap()).unwrap();

        assert_eq!(decoded.num_required_signatures(), 1);
        assert_eq!(decoded.get_compute_unit_limit(), Some(DEFAULT_SWAP_GAS_LIMIT));
        assert_eq!(decoded.get_compute_unit_price(), Some(0));
    }
}
