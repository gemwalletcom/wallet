use gem_solana::{Instruction, JUPITER_PROGRAM_ID, Pubkey, anchor::global_discriminator};

const AMOUNT_LENGTH: usize = 8;

enum DestinationAccount {
    Required(usize),
    Optional { account: usize, user_account: usize },
}

struct RouteLayout {
    name: &'static str,
    authority: usize,
    source: usize,
    destination: DestinationAccount,
    in_amount: usize,
}

const ROUTE_LAYOUTS: [RouteLayout; 2] = [
    RouteLayout {
        name: "route_v2",
        authority: 0,
        source: 1,
        destination: DestinationAccount::Optional { account: 7, user_account: 2 },
        in_amount: 8,
    },
    RouteLayout {
        name: "shared_accounts_route_v2",
        authority: 1,
        source: 2,
        destination: DestinationAccount::Required(5),
        in_amount: 9,
    },
];

#[derive(Debug, PartialEq, Eq)]
pub(super) struct JupiterRoute {
    pub authority: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub in_amount: u64,
}

impl JupiterRoute {
    pub(super) fn decode(instruction: &Instruction) -> Option<Self> {
        if instruction.program_id.to_base58() != JUPITER_PROGRAM_ID {
            return None;
        }
        let layout = ROUTE_LAYOUTS.iter().find(|layout| instruction.data.starts_with(&global_discriminator(layout.name)))?;
        let account = |index: usize| instruction.accounts.get(index).map(|account| account.pubkey);
        let destination = match layout.destination {
            DestinationAccount::Required(index) => account(index)?,
            DestinationAccount::Optional { account: index, user_account } => match account(index)? {
                placeholder if placeholder == instruction.program_id => account(user_account)?,
                destination => destination,
            },
        };
        let in_amount = instruction.data.get(layout.in_amount..layout.in_amount + AMOUNT_LENGTH)?.try_into().ok().map(u64::from_le_bytes)?;
        Some(Self {
            authority: account(layout.authority)?,
            source: account(layout.source)?,
            destination,
            in_amount,
        })
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
                in_amount: 100_000_000,
            }),
            "a route without its own destination pays into the user's destination account"
        );
        assert_eq!(
            JupiterRoute::decode(&shared_accounts_route),
            Some(JupiterRoute {
                authority: wallet,
                source: usdc,
                destination: wrapped_sol,
                in_amount: 20_000_000,
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
        let short_data = Instruction {
            data: route.data[..12].to_vec(),
            ..route.clone()
        };
        assert_eq!(JupiterRoute::decode(&other_program), None);
        assert_eq!(JupiterRoute::decode(&other_instruction), None, "a route whose amount sits behind its route plan is refused");
        assert_eq!(JupiterRoute::decode(&short_accounts), None);
        assert_eq!(JupiterRoute::decode(&short_data), None);
    }
}
