use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Instruction, JUPITER_PROGRAM_ID, Pubkey, anchor::instruction_arguments};

use crate::fees::subtract_bps;

const ROUTE: &str = "route_v2";
const SHARED_ACCOUNTS_ROUTE: &str = "shared_accounts_route_v2";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RouteArguments {
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u16,
}

impl BorshDeserialize for RouteArguments {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            in_amount: reader.read()?,
            quoted_out_amount: reader.read()?,
            slippage_bps: reader.read()?,
            platform_fee_bps: reader.read()?,
        })
    }
}

impl RouteArguments {
    pub(super) fn minimum_out(&self) -> u64 {
        subtract_bps(&self.quoted_out_amount, u32::from(self.slippage_bps))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct JupiterRoute {
    pub authority: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub arguments: RouteArguments,
}

impl JupiterRoute {
    pub(super) fn decode(instruction: &Instruction) -> Option<Self> {
        if instruction.program_id.to_base58() != JUPITER_PROGRAM_ID {
            return None;
        }
        if let Ok(arguments) = instruction_arguments::<RouteArguments>(&instruction.data, ROUTE) {
            let destination = match instruction.account(7)? {
                placeholder if *placeholder == instruction.program_id => instruction.account(2)?,
                destination => destination,
            };
            return Some(Self {
                authority: *instruction.account(0)?,
                source: *instruction.account(1)?,
                destination: *destination,
                arguments,
            });
        }
        let (_, arguments) = instruction_arguments::<(u8, RouteArguments)>(&instruction.data, SHARED_ACCOUNTS_ROUTE).ok()?;
        Some(Self {
            authority: *instruction.account(1)?,
            source: *instruction.account(2)?,
            destination: *instruction.account(5)?,
            arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jupiter::{model::BuildResponse, testkit::TEST_PAYER};
    use gem_solana::{
        AccountMeta, Base64InstructionData, USDC_TOKEN_MINT, anchor::global_discriminator, associated_token::get_associated_token_address_with_program_id, decode_instruction, instructions::program_ids::token_program, wrapped_sol,
    };

    #[test]
    fn test_decode() {
        let wallet = Pubkey::from_base58(TEST_PAYER).unwrap();
        let usdc = get_associated_token_address_with_program_id(&wallet, &Pubkey::from_base58(USDC_TOKEN_MINT).unwrap(), &token_program()).unwrap();
        let wrapped_sol = wrapped_sol::wrapped_sol_account(&wallet).unwrap();
        let route = decode_instruction::<Base64InstructionData>(BuildResponse::mock().swap_instruction).unwrap();
        let shared_accounts_route = decode_instruction::<Base64InstructionData>(BuildResponse::mock_usdc_sol().swap_instruction).unwrap();

        assert_eq!(
            JupiterRoute::decode(&route),
            Some(JupiterRoute {
                authority: wallet,
                source: wrapped_sol,
                destination: usdc,
                arguments: RouteArguments {
                    in_amount: 100_000_000,
                    quoted_out_amount: 6_829_563,
                    slippage_bps: 100,
                    platform_fee_bps: 50,
                },
            }),
            "a route without its own destination pays into the user's destination account"
        );
        assert_eq!(
            JupiterRoute::decode(&shared_accounts_route),
            Some(JupiterRoute {
                authority: wallet,
                source: usdc,
                destination: wrapped_sol,
                arguments: RouteArguments {
                    in_amount: 20_000_000,
                    quoted_out_amount: 289_924_882,
                    slippage_bps: 100,
                    platform_fee_bps: 50,
                },
            })
        );

        let mut explicit_destination = route.clone();
        explicit_destination.accounts[7] = AccountMeta::new_writable(wrapped_sol);
        assert_eq!(JupiterRoute::decode(&explicit_destination).map(|route| route.destination), Some(wrapped_sol));

        let other_program = Instruction {
            program_id: token_program(),
            ..route.clone()
        };
        let other_instruction = Instruction {
            data: [global_discriminator("route").as_slice(), &route.data[8..]].concat(),
            ..route.clone()
        };
        let short_accounts = Instruction {
            accounts: route.accounts[..7].to_vec(),
            ..route.clone()
        };
        let short_data = Instruction { data: route.data[..12].to_vec(), ..route };
        assert_eq!(JupiterRoute::decode(&other_program), None);
        assert_eq!(JupiterRoute::decode(&other_instruction), None, "a route whose amount sits behind its route plan is refused");
        assert_eq!(JupiterRoute::decode(&short_accounts), None);
        assert_eq!(JupiterRoute::decode(&short_data), None);
    }
}
