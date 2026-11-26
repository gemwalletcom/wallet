use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{
    HexInstructionData, Instruction, Pubkey, StandardInstruction,
    anchor::{global_discriminator, instruction_arguments},
    decode_instructions, wrapped_sol,
};
use num_bigint::BigUint;
use primitives::{
    Chain,
    contract_constants::{SOLANA_JUPITER_PROGRAM_ID, SOLANA_RELAY_DEPOSITORY_PROGRAM_ID, SOLANA_RELAY_FORWARDER_PROGRAM_ID},
};

use super::{
    contracts::contracts,
    model::{RelayQuoteResponse, SolanaStepData},
};
use crate::{QuoteRequest, SwapperError, binding_check::solana::verify_steps, jupiter::JupiterRoute, solana_dex::SolanaTransaction};

const DEPOSIT_NATIVE: &str = "deposit_native";
const DEPOSIT_TOKEN: &str = "deposit_token";
const FORWARD_TOKEN: &str = "forward_token";
const DEPOSIT_TOKEN_MINT: usize = 4;
const FORWARDER_TOKEN_ACCOUNT: usize = 6;

struct DepositArguments {
    amount: u64,
}

impl BorshDeserialize for DepositArguments {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self { amount: reader.read()? })
    }
}

struct QuotedDeposit {
    wallet: Pubkey,
    mint: Option<Pubkey>,
    amount: u64,
}

enum RelayStep<'a> {
    Deposit { mint: Option<&'a Pubkey>, amount: u64 },
    Forward { token_account: &'a Pubkey },
    Swap(JupiterRoute),
    Other,
    Unsupported,
}

pub fn quoted_transaction(request: &QuoteRequest, value: &BigUint, response: &RelayQuoteResponse) -> Result<SolanaTransaction, SwapperError> {
    transaction(response.get_solana_step().ok_or(SwapperError::InvalidRoute)?, &QuotedDeposit::new(request, value)?)
}

fn transaction(step: &SolanaStepData, quoted: &QuotedDeposit) -> Result<SolanaTransaction, SwapperError> {
    let instructions = decode_instructions::<HexInstructionData>(step.instructions.clone()).map_err(SwapperError::transaction_error)?;
    let steps: Vec<StandardInstruction> = instructions.iter().map(StandardInstruction::new).collect();
    verify_steps(&steps, &quoted.wallet, &contracts(Chain::Solana).targets)?;
    quoted.verify(&instructions.iter().map(RelayStep::decode).collect::<Vec<_>>())?;
    Ok(SolanaTransaction::new(instructions, step.address_lookup_table_addresses.clone()))
}

impl QuotedDeposit {
    fn new(request: &QuoteRequest, value: &BigUint) -> Result<Self, SwapperError> {
        Ok(Self {
            wallet: Pubkey::from_base58(&request.wallet_address).map_err(SwapperError::transaction_error)?,
            mint: request.from_asset.asset_id().token_id.as_deref().map(Pubkey::from_base58).transpose().map_err(SwapperError::transaction_error)?,
            amount: u64::try_from(value).map_err(SwapperError::transaction_error)?,
        })
    }

    fn verify(&self, steps: &[RelayStep]) -> Result<(), SwapperError> {
        let input_mint = self.mint.unwrap_or_else(wrapped_sol::wrapped_sol_mint);
        let spent = steps.iter().try_fold(0u64, |total, step| match step {
            RelayStep::Deposit { mint, amount } if *mint == self.mint.as_ref() => Ok(total.saturating_add(*amount)),
            RelayStep::Swap(route) if route.spends_from(&self.wallet, &input_mint) && steps.iter().any(|forward| forward.forwards(&route.destination)) => Ok(total.saturating_add(route.arguments.in_amount)),
            RelayStep::Forward { .. } | RelayStep::Other => Ok(total),
            RelayStep::Deposit { .. } | RelayStep::Swap(_) | RelayStep::Unsupported => Err(SwapperError::InvalidRoute),
        })?;
        if spent > self.amount {
            return Err(SwapperError::InvalidRoute);
        }
        Ok(())
    }
}

impl<'a> RelayStep<'a> {
    fn decode(instruction: &'a Instruction) -> Self {
        let deposit = |name| instruction_arguments::<DepositArguments>(&instruction.data, name).map(|arguments| arguments.amount);
        match instruction.program_id.to_base58().as_str() {
            SOLANA_RELAY_DEPOSITORY_PROGRAM_ID => match (deposit(DEPOSIT_NATIVE), deposit(DEPOSIT_TOKEN), instruction.account(DEPOSIT_TOKEN_MINT)) {
                (Ok(amount), _, _) => Self::Deposit { mint: None, amount },
                (_, Ok(amount), Some(mint)) => Self::Deposit { mint: Some(mint), amount },
                _ => Self::Unsupported,
            },
            SOLANA_RELAY_FORWARDER_PROGRAM_ID => match (instruction.data.starts_with(&global_discriminator(FORWARD_TOKEN)), instruction.account(FORWARDER_TOKEN_ACCOUNT)) {
                (true, Some(token_account)) => Self::Forward { token_account },
                _ => Self::Unsupported,
            },
            SOLANA_JUPITER_PROGRAM_ID => JupiterRoute::decode(instruction).map_or(Self::Unsupported, Self::Swap),
            _ => Self::Other,
        }
    }

    fn forwards(&self, account: &Pubkey) -> bool {
        match self {
            Self::Forward { token_account } => *token_account == account,
            Self::Deposit { .. } | Self::Swap(_) | Self::Other | Self::Unsupported => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, binding_check::solana::verify_solana, relay::testkit::TEST_SOLANA_PAYER, testkit::mock_quote};
    use gem_solana::{AddressLookupTableAccount, decode_transaction};
    use primitives::{
        AssetId,
        asset_constants::{BASE_USDC_ASSET_ID, SOLANA_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID, SOLANA_USDT_ASSET_ID},
        hex,
        testkit::signer_mock::TEST_PRIVATE_KEY_SOLANA_ADDRESS,
    };

    #[test]
    fn test_build_transaction() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_sol_to_base_usdc.json")).unwrap();
        let step = response.get_solana_step().unwrap();
        let request = QuoteRequest {
            wallet_address: "4Nd1mBQtrMJVYVfKf2PJy9NZUZdTAsp7D4xWLs4gDB4T".into(),
            value: BigUint::from(100_000_000u64),
            ..mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)), SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()))
        };

        let quote_data = quoted_transaction(&request, &request.value, &response).unwrap().encode(&request.wallet_address, &[]).unwrap();
        verify_solana(&quote_data.data, &request.wallet_address, &contracts(Chain::Solana).targets).unwrap();

        let transaction_data = decode_transaction(&quote_data.data).unwrap();
        assert_eq!(transaction_data.num_required_signatures(), 1);
        assert_eq!(transaction_data.get_compute_unit_price(), Some(0));
        assert!(quote_data.to.is_empty());
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert!(quote_data.approval.is_none());

        let instructions = decode_instructions::<HexInstructionData>(step.instructions.clone()).unwrap();
        assert_eq!(instructions[0].program_id.to_base58(), SOLANA_RELAY_DEPOSITORY_PROGRAM_ID);
        assert_eq!(instructions[0].data, hex::decode("0d9e0ddf5fd51c0600e1f50500000000d8f6831d9a771a5b031cc86256987f54a9759e6107db24b07ec66eef4e555055").unwrap());

        let mut other_instruction = step.clone();
        other_instruction.instructions[0].data = hex::encode([global_discriminator("withdraw").as_slice(), &instructions[0].data[8..]].concat());
        assert_eq!(
            transaction(&other_instruction, &QuotedDeposit::new(&request, &request.value).unwrap()).err(),
            Some(SwapperError::InvalidRoute),
            "an instruction of Relay's programs the check does not read"
        );
        for (request, reason) in [
            (
                QuoteRequest {
                    value: BigUint::from(99_999_999u64),
                    ..request.clone()
                },
                "a deposit larger than the quote",
            ),
            (
                QuoteRequest {
                    from_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
                    ..request
                },
                "a deposit of another asset",
            ),
        ] {
            assert_eq!(quoted_transaction(&request, &request.value, &response).err(), Some(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_build_token_deposit() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_usdt_to_base_usdc.json")).unwrap();
        let request = QuoteRequest {
            wallet_address: TEST_SOLANA_PAYER.into(),
            value: BigUint::from(20_000_000u64),
            ..mock_quote(SwapperQuoteAsset::from(SOLANA_USDT_ASSET_ID.clone()), SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()))
        };

        assert_eq!(quoted_transaction(&request, &request.value, &response).err(), None);
        for (request, reason) in [
            (
                QuoteRequest {
                    value: BigUint::from(19_999_999u64),
                    ..request.clone()
                },
                "a deposit larger than the quote",
            ),
            (
                QuoteRequest {
                    from_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
                    ..request
                },
                "a deposit of another token",
            ),
        ] {
            assert_eq!(quoted_transaction(&request, &request.value, &response).err(), Some(SwapperError::InvalidRoute), "{reason}");
        }
    }

    #[test]
    fn test_build_transaction_through_jupiter() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_jup_to_base_usdc.json")).unwrap();
        let step = response.get_solana_step().unwrap();
        let instructions = decode_instructions::<HexInstructionData>(step.instructions.clone()).unwrap();
        let mut accounts = Vec::new();
        for account in instructions.iter().flat_map(|instruction| &instruction.accounts) {
            if !account.is_signer && !accounts.contains(&account.pubkey) {
                accounts.push(account.pubkey);
            }
        }
        let lookup_table = AddressLookupTableAccount::new(Pubkey::new([7; 32]), accounts);
        let request = QuoteRequest {
            wallet_address: TEST_SOLANA_PAYER.into(),
            value: BigUint::from(50_000_000u64),
            ..mock_quote(
                SwapperQuoteAsset::from(AssetId::from_token(Chain::Solana, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN")),
                SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            )
        };
        let quoted = QuotedDeposit::new(&request, &request.value).unwrap();

        let quote_data = transaction(step, &quoted).unwrap().encode(TEST_SOLANA_PAYER, &[lookup_table]).unwrap();
        verify_solana(&quote_data.data, TEST_SOLANA_PAYER, &contracts(Chain::Solana).targets).unwrap();

        let route = JupiterRoute::decode(&instructions[4]).unwrap();
        let mut to_user_destination = instructions[4].clone();
        to_user_destination.accounts[4].pubkey = to_user_destination.program_id;
        assert_eq!(route.arguments.in_amount, 50_000_000);
        assert_eq!(Some(&route.destination), instructions[5].account(FORWARDER_TOKEN_ACCOUNT));
        assert_eq!(JupiterRoute::decode(&to_user_destination).map(|route| route.destination), to_user_destination.account(3).copied());

        let mut other_token_program = step.clone();
        other_token_program.instructions[1].accounts[5].pubkey = TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string();
        let mut other_destination = step.clone();
        other_destination.instructions[4].accounts[4].pubkey = TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string();
        let mut other_authority = step.clone();
        other_authority.instructions[4].accounts[1].pubkey = TEST_PRIVATE_KEY_SOLANA_ADDRESS.to_string();
        let with_parameters: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_cbbtc_to_base_usdc.json")).unwrap();
        let cbbtc = QuoteRequest {
            value: BigUint::from(20_000_000u64),
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Solana, "cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij")),
            ..request.clone()
        };
        for (step, quoted, reason) in [
            (&other_token_program, &quoted, "the steps are checked before a lookup table can hide an account"),
            (step, &QuotedDeposit { amount: 49_999_999, ..quoted }, "a route that spends more than quoted"),
            (
                step,
                &QuotedDeposit {
                    mint: Some(Pubkey::from_base58(SOLANA_USDC_TOKEN_ID).unwrap()),
                    ..quoted
                },
                "a route that spends another token",
            ),
            (&other_destination, &quoted, "a route that pays into an account Relay's forwarder does not deposit"),
            (&other_authority, &quoted, "a route another account signs"),
            (
                with_parameters.get_solana_step().unwrap(),
                &QuotedDeposit::new(&cbbtc, &cbbtc.value).unwrap(),
                "a route whose steps carry parameters, so the amount it spends cannot be read",
            ),
        ] {
            assert_eq!(transaction(step, quoted).err(), Some(SwapperError::InvalidRoute), "{reason}");
        }
    }
}
