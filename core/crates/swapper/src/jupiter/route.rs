use gem_encoding::borsh::{BorshDeserialize, DecodeError, Reader};
use gem_solana::{Instruction, JUPITER_PROGRAM_ID, Pubkey, anchor::global_discriminator, associated_token::is_associated_token_address};

use crate::fees::subtract_bps;

type UnitRouteStep = [u8; 4];

struct RouteLayout {
    instruction: &'static str,
    authority: usize,
    source: usize,
    destination: usize,
    user_destination: Option<usize>,
    arguments: fn(&[u8]) -> Option<RouteArguments>,
}

const ROUTE_LAYOUTS: [RouteLayout; 3] = [
    RouteLayout {
        instruction: "route_v2",
        authority: 0,
        source: 1,
        destination: 7,
        user_destination: Some(2),
        arguments: RouteArguments::route,
    },
    RouteLayout {
        instruction: "shared_accounts_route_v2",
        authority: 1,
        source: 2,
        destination: 5,
        user_destination: None,
        arguments: RouteArguments::shared_accounts_route,
    },
    RouteLayout {
        instruction: "route",
        authority: 1,
        source: 2,
        destination: 4,
        user_destination: Some(3),
        arguments: RouteArguments::unit_steps_route,
    },
];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RouteArguments {
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

    fn route(mut body: &[u8]) -> Option<Self> {
        Self::deserialize(&mut body).ok()
    }

    fn shared_accounts_route(mut body: &[u8]) -> Option<Self> {
        <(u8, Self)>::deserialize(&mut body).ok().map(|(_, arguments)| arguments)
    }

    fn unit_steps_route(body: &[u8]) -> Option<Self> {
        UnitStepsRoute::try_from_slice(body).ok().map(|UnitStepsRoute(arguments)| arguments)
    }
}

struct UnitStepsRoute(RouteArguments);

impl BorshDeserialize for UnitStepsRoute {
    fn deserialize_reader(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.read::<Vec<UnitRouteStep>>()?;
        Ok(Self(RouteArguments {
            in_amount: reader.read()?,
            quoted_out_amount: reader.read()?,
            slippage_bps: reader.read()?,
            platform_fee_bps: u16::from(reader.read::<u8>()?),
        }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct JupiterRoute {
    pub authority: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub arguments: RouteArguments,
}

impl JupiterRoute {
    pub(crate) fn decode(instruction: &Instruction) -> Option<Self> {
        if instruction.program_id.to_base58() != JUPITER_PROGRAM_ID {
            return None;
        }
        let (layout, body) = ROUTE_LAYOUTS
            .iter()
            .find_map(|layout| instruction.data.strip_prefix(global_discriminator(layout.instruction).as_slice()).map(|body| (layout, body)))?;
        let destination = match (instruction.account(layout.destination)?, layout.user_destination) {
            (placeholder, Some(user_destination)) if *placeholder == instruction.program_id => instruction.account(user_destination)?,
            (destination, _) => destination,
        };
        Some(Self {
            authority: *instruction.account(layout.authority)?,
            source: *instruction.account(layout.source)?,
            destination: *destination,
            arguments: (layout.arguments)(body)?,
        })
    }

    pub(crate) fn spends_from(&self, wallet: &Pubkey, mint: &Pubkey) -> bool {
        self.authority == *wallet && is_associated_token_address(&self.source, wallet, mint)
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
        let short_accounts = Instruction {
            accounts: route.accounts[..7].to_vec(),
            ..route.clone()
        };
        let short_data = Instruction { data: route.data[..12].to_vec(), ..route };
        assert_eq!(JupiterRoute::decode(&other_program), None);
        assert_eq!(JupiterRoute::decode(&short_accounts), None);
        assert_eq!(JupiterRoute::decode(&short_data), None);
    }

    #[test]
    fn test_decode_unit_steps_route() {
        let route = Instruction {
            data: [
                global_discriminator("route").as_slice(),
                &1u32.to_le_bytes(),
                &[195, 100, 0, 1],
                &100_000_000u64.to_le_bytes(),
                &6_829_563u64.to_le_bytes(),
                &100u16.to_le_bytes(),
                &[50],
            ]
            .concat(),
            ..decode_instruction::<Base64InstructionData>(BuildResponse::mock().swap_instruction).unwrap()
        };
        let trailing = Instruction {
            data: [route.data.as_slice(), &[0]].concat(),
            ..route.clone()
        };

        assert_eq!(
            JupiterRoute::decode(&route).map(|route| route.arguments),
            Some(RouteArguments {
                in_amount: 100_000_000,
                quoted_out_amount: 6_829_563,
                slippage_bps: 100,
                platform_fee_bps: 50,
            })
        );
        assert_eq!(JupiterRoute::decode(&trailing), None, "Jupiter ignores bytes after its arguments, so they could hide the amount it reads");
    }
}
