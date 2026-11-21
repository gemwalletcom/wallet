use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{
    Instruction, JUPITER_PROGRAM_ID, Pubkey,
    anchor::{global_discriminator, instruction_arguments},
    associated_token::is_associated_token_address,
};

use crate::fees::subtract_bps;

const ROUTE: &str = "route_v2";
const SHARED_ACCOUNTS_ROUTE: &str = "shared_accounts_route_v2";

struct RouteLayout {
    instruction: &'static str,
    authority: usize,
    source: usize,
    destination: usize,
    user_destination: Option<usize>,
}

const ROUTE_LAYOUTS: [RouteLayout; 3] = [
    RouteLayout {
        instruction: ROUTE,
        authority: 0,
        source: 1,
        destination: 7,
        user_destination: Some(2),
    },
    RouteLayout {
        instruction: SHARED_ACCOUNTS_ROUTE,
        authority: 1,
        source: 2,
        destination: 5,
        user_destination: None,
    },
    RouteLayout {
        instruction: "route",
        authority: 1,
        source: 2,
        destination: 4,
        user_destination: Some(3),
    },
];

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
pub(crate) struct RouteAccounts {
    pub authority: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
}

impl RouteAccounts {
    pub(crate) fn decode(instruction: &Instruction) -> Option<Self> {
        if instruction.program_id.to_base58() != JUPITER_PROGRAM_ID {
            return None;
        }
        let layout = ROUTE_LAYOUTS.iter().find(|layout| instruction.data.starts_with(&global_discriminator(layout.instruction)))?;
        let destination = match (instruction.account(layout.destination)?, layout.user_destination) {
            (placeholder, Some(user_destination)) if *placeholder == instruction.program_id => instruction.account(user_destination)?,
            (destination, _) => destination,
        };
        Some(Self {
            authority: *instruction.account(layout.authority)?,
            source: *instruction.account(layout.source)?,
            destination: *destination,
        })
    }

    pub(crate) fn spends_from(&self, wallet: &Pubkey, mint: &Pubkey) -> bool {
        self.authority == *wallet && is_associated_token_address(&self.source, wallet, mint)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct JupiterRoute {
    pub accounts: RouteAccounts,
    pub arguments: RouteArguments,
}

impl JupiterRoute {
    pub(super) fn decode(instruction: &Instruction) -> Option<Self> {
        let accounts = RouteAccounts::decode(instruction)?;
        let arguments = match instruction_arguments::<RouteArguments>(&instruction.data, ROUTE) {
            Ok(arguments) => arguments,
            Err(_) => instruction_arguments::<(u8, RouteArguments)>(&instruction.data, SHARED_ACCOUNTS_ROUTE).ok()?.1,
        };
        Some(Self { accounts, arguments })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jupiter::{model::BuildResponse, testkit::TEST_PAYER};
    use gem_solana::{AccountMeta, Base64InstructionData, USDC_TOKEN_MINT, associated_token::get_associated_token_address_with_program_id, decode_instruction, instructions::program_ids::token_program, wrapped_sol};

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
                accounts: RouteAccounts {
                    authority: wallet,
                    source: wrapped_sol,
                    destination: usdc,
                },
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
                accounts: RouteAccounts {
                    authority: wallet,
                    source: usdc,
                    destination: wrapped_sol,
                },
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
        assert_eq!(RouteAccounts::decode(&explicit_destination).map(|accounts| accounts.destination), Some(wrapped_sol));

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
