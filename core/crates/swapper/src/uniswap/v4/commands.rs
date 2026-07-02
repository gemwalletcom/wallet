use std::str::FromStr;

use crate::{QuoteRequest, Route, SwapperError, eth_address, fees::default_referral_fees, uniswap::requires_native_wrapping};
use alloy_primitives::{Address, U256};
use gem_evm::uniswap::{
    actions::V4Action::{SETTLE, SWAP_EXACT_IN, SWAP_EXACT_IN_V2_1, TAKE},
    command::{ADDRESS_THIS, PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand},
    contracts::v4::{
        IV4Router::{ExactInputParams, ExactInputParamsV2_1},
        PathKey,
    },
    deployment::UniversalRouterAbi,
};

pub fn build_commands(
    request: &QuoteRequest,
    token_in: &Address,
    token_out: &Address,
    amount_in: u128,
    quote_amount: u128,
    swap_routes: &[Route],
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
    universal_router_abi: UniversalRouterAbi,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let fee_options = default_referral_fees().evm;
    let recipient = eth_address::parse_str(&request.wallet_address)?;

    let input_is_native = requires_native_wrapping(&request.from_asset.asset_id());
    let pay_fees = fee_options.bps > 0;

    let mut commands: Vec<UniversalRouterCommand> = vec![];

    let amount_out = quote_amount;
    // Insert permit2 if needed
    if let Some(permit) = permit {
        commands.push(UniversalRouterCommand::PERMIT2_PERMIT(permit));
    }

    if pay_fees {
        if fee_token_is_input {
            // insert TRANSFER fee first
            let fee = amount_in * (fee_options.bps as u128) / 10000_u128;
            let fee_recipient = Address::from_str(fee_options.address.as_str()).unwrap();
            if input_is_native {
                // if input is native ETH, we can transfer directly
                commands.push(UniversalRouterCommand::TRANSFER(Transfer {
                    token: *token_in,
                    recipient: fee_recipient,
                    value: U256::from(fee),
                }));
            } else {
                // call permit2 transfer instead
                commands.push(UniversalRouterCommand::PERMIT2_TRANSFER_FROM(Transfer {
                    token: *token_in,
                    recipient: fee_recipient,
                    value: U256::from(fee),
                }));
            };
            // insert V4_SWAP with amount - fee
            // fee charged in token_in, so we need to use recipient as recipient
            let command = build_v4_swap_command(token_in, token_out, amount_in - fee, amount_out, swap_routes, &recipient, universal_router_abi)?;
            commands.push(command);
        } else {
            // insert V4 SWAP
            // if needs to pay fees, amount_out_min set to 0 and we will sweep the rest
            let address_this = ADDRESS_THIS.parse().unwrap();
            let amount_out_min = if pay_fees { 0 } else { amount_out };
            let command = build_v4_swap_command(token_in, token_out, amount_in, amount_out_min, swap_routes, &address_this, universal_router_abi)?;
            commands.push(command);

            // insert PAY_PORTION to fee_address
            commands.push(UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: *token_out,
                recipient: Address::from_str(fee_options.address.as_str()).unwrap(),
                bips: U256::from(fee_options.bps),
            }));

            commands.push(UniversalRouterCommand::SWEEP(Sweep {
                token: *token_out,
                recipient,
                amount_min: U256::from(amount_out),
            }));
        }
    } else {
        let command = build_v4_swap_command(token_in, token_out, amount_in, amount_out, swap_routes, &recipient, universal_router_abi)?;
        commands.push(command);
    }
    Ok(commands)
}

fn build_v4_swap_command(
    token_in: &Address,
    token_out: &Address,
    amount_in: u128,
    amount_out_min: u128,
    swap_routes: &[Route],
    recipient: &Address,
    universal_router_abi: UniversalRouterAbi,
) -> Result<UniversalRouterCommand, SwapperError> {
    if swap_routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    // V4_SWAP {actions}
    // Dispatcher -> BaseActionsRouter::_executeActions -> PoolManager::_executeActionsWithoutUnlock -> V4Router::_handleAction
    let path: Vec<PathKey> = swap_routes
        .iter()
        .map(|route| PathKey::try_from(route).map_err(|_| SwapperError::InvalidRoute))
        .collect::<Result<Vec<PathKey>, SwapperError>>()?;
    let swap_action = match universal_router_abi {
        UniversalRouterAbi::V2 => SWAP_EXACT_IN(ExactInputParams {
            currencyIn: *token_in,
            path,
            amountIn: amount_in,
            amountOutMinimum: amount_out_min,
        }),
        UniversalRouterAbi::V2_1 => SWAP_EXACT_IN_V2_1(ExactInputParamsV2_1 {
            currencyIn: *token_in,
            path,
            minHopPriceX36: vec![],
            amountIn: amount_in,
            amountOutMinimum: amount_out_min,
        }),
    };
    let actions = vec![
        swap_action,
        SETTLE {
            currency: *token_in,
            amount: U256::from(0),
            payer_is_user: true,
        },
        TAKE {
            currency: *token_out,
            recipient: recipient.to_owned(),
            amount: U256::from(0),
        },
    ];
    Ok(UniversalRouterCommand::V4_SWAP { actions })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Options;
    use gem_evm::uniswap::deployment::UniversalRouterAbi;
    use primitives::{
        AssetId, Chain,
        asset_constants::{CELO_USDT_TOKEN_ID, CELO_WETH_TOKEN_ID},
    };

    #[test]
    fn test_build_commands_celo_tokenized_native() {
        let token_celo = Address::from_str(CELO_WETH_TOKEN_ID).unwrap();
        let token_usdt = Address::from_str(CELO_USDT_TOKEN_ID).unwrap();
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(
            AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())),
            AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())),
        )];

        // CELO -> USDT: no wrap, direct swap through token path
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "22000000000000000000".into(),
            options: Options::default(),
        };
        let commands = build_commands(
            &request,
            &token_celo,
            &token_usdt,
            22_000_000_000_000_000_000,
            14_804_757,
            &routes,
            None,
            false,
            UniversalRouterAbi::V2,
        )
        .unwrap();

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], UniversalRouterCommand::V4_SWAP { .. }));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));

        let UniversalRouterCommand::SWEEP(sweep) = &commands[2] else {
            panic!("expected SWEEP command");
        };
        assert_eq!(sweep.amount_min, U256::from(14_804_757u64));

        // USDT -> CELO with fees: sweep instead of unwrap
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Celo, None).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "900000".into(),
            options: Options {
                slippage: 50.into(),
                use_max_amount: false,
            },
        };
        let routes = vec![Route::mock(
            AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())),
            AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())),
        )];
        let commands = build_commands(
            &request,
            &token_usdt,
            &token_celo,
            900_000,
            10_752_991_111_111_111_170,
            &routes,
            None,
            false,
            UniversalRouterAbi::V2,
        )
        .unwrap();

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], UniversalRouterCommand::V4_SWAP { .. }));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));
    }

    #[test]
    fn test_build_commands_v2_1_uses_v2_1_swap_action() {
        let token_celo = Address::from_str(CELO_WETH_TOKEN_ID).unwrap();
        let token_usdt = Address::from_str(CELO_USDT_TOKEN_ID).unwrap();
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(
            AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())),
            AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())),
        )];
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: "22000000000000000000".into(),
            options: Options::default(),
        };
        let commands = build_commands(
            &request,
            &token_celo,
            &token_usdt,
            22_000_000_000_000_000_000,
            14_804_757,
            &routes,
            None,
            false,
            UniversalRouterAbi::V2_1,
        )
        .unwrap();

        match &commands[0] {
            UniversalRouterCommand::V4_SWAP { actions } => assert!(matches!(actions[0], SWAP_EXACT_IN_V2_1(_))),
            _ => panic!("expected V4 swap command"),
        }
    }
}
