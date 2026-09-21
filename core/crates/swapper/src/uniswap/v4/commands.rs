use crate::{
    QuoteRequest, Route, SwapperError, eth_address,
    fees::default_referral_fees,
    uniswap::routed_asset::{Funding, RoutedAsset},
};
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
    input: &RoutedAsset,
    output: &RoutedAsset,
    amount_in: u128,
    quote_amount: u128,
    swap_routes: &[Route],
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
    universal_router_abi: UniversalRouterAbi,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let fee_options = default_referral_fees().evm;
    let recipient = eth_address::parse_str(&request.wallet_address)?;

    let token_in = input.address;
    let token_out = output.address;
    let payer_is_user = input.funding == Funding::Permit2;
    let pay_fees = fee_options.bps > 0;

    let mut commands: Vec<UniversalRouterCommand> = vec![];

    let amount_out = quote_amount;
    if payer_is_user && let Some(permit) = permit {
        commands.push(UniversalRouterCommand::PERMIT2_PERMIT(permit));
    }

    if pay_fees {
        if fee_token_is_input {
            let fee = amount_in * (fee_options.bps as u128) / 10000_u128;
            let fee_transfer = Transfer {
                token: token_in,
                recipient: eth_address::parse_str(&fee_options.address)?,
                value: U256::from(fee),
            };
            commands.push(match input.funding {
                Funding::Value | Funding::RouterBalance => UniversalRouterCommand::TRANSFER(fee_transfer),
                Funding::Permit2 => UniversalRouterCommand::PERMIT2_TRANSFER_FROM(fee_transfer),
            });
            let command = build_v4_swap_command(&token_in, &token_out, amount_in - fee, amount_out, swap_routes, &recipient, payer_is_user, universal_router_abi)?;
            commands.push(command);
        } else {
            let address_this = eth_address::parse_str(ADDRESS_THIS)?;
            let command = build_v4_swap_command(&token_in, &token_out, amount_in, 0, swap_routes, &address_this, payer_is_user, universal_router_abi)?;
            commands.push(command);

            commands.push(UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: token_out,
                recipient: eth_address::parse_str(&fee_options.address)?,
                bips: U256::from(fee_options.bps),
            }));

            commands.push(UniversalRouterCommand::SWEEP(Sweep {
                token: token_out,
                recipient,
                amount_min: U256::from(amount_out),
            }));
        }
    } else {
        let command = build_v4_swap_command(&token_in, &token_out, amount_in, amount_out, swap_routes, &recipient, payer_is_user, universal_router_abi)?;
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
    payer_is_user: bool,
    universal_router_abi: UniversalRouterAbi,
) -> Result<UniversalRouterCommand, SwapperError> {
    if swap_routes.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
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
            payer_is_user,
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
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, CELO_USDT_TOKEN_ID, CELO_WETH_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID},
    };
    use std::str::FromStr;

    #[test]
    fn test_build_commands_router_balance_input() {
        let usdc = Address::from_str(ARC_USDC_TOKEN_ID).unwrap();
        let eurc = Address::from_str(ARC_EURC_TOKEN_ID).unwrap();
        let input = RoutedAsset::mock_router_balance(ARC_USDC_TOKEN_ID, 1_000_000_000_000);
        let output = RoutedAsset::mock_permit2(ARC_EURC_TOKEN_ID);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(AssetId::from(Chain::Arc, Some(ARC_USDC_TOKEN_ID.into())), AssetId::from(Chain::Arc, Some(ARC_EURC_TOKEN_ID.into())))];
        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Arc).into(),
            to_asset: AssetId::from(Chain::Arc, Some(ARC_EURC_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(3_000_000_000_000_000_000u64),
            options: Options::default(),
        };

        let commands = build_commands(&request, &input, &output, 3_000_000, 2_523_162, &routes, None, true, UniversalRouterAbi::V2_1).unwrap();

        assert_eq!(commands.len(), 2);
        let UniversalRouterCommand::TRANSFER(fee) = &commands[0] else {
            panic!("expected TRANSFER fee from the router balance");
        };
        assert_eq!(fee.token, usdc);
        let UniversalRouterCommand::V4_SWAP { actions } = &commands[1] else {
            panic!("expected V4_SWAP");
        };
        assert!(matches!(actions[0], SWAP_EXACT_IN_V2_1(_)));
        assert!(matches!(&actions[1], SETTLE { currency, payer_is_user: false, .. } if *currency == usdc));
        assert!(matches!(&actions[2], TAKE { currency, .. } if *currency == eurc));

        let commands = build_commands(&request, &input, &output, 3_000_000, 2_523_162, &routes, None, false, UniversalRouterAbi::V2_1).unwrap();
        let UniversalRouterCommand::V4_SWAP { actions } = &commands[0] else {
            panic!("expected V4_SWAP");
        };
        assert!(matches!(&actions[1], SETTLE { payer_is_user: false, .. }));
    }

    #[test]
    fn test_build_commands_native_value_input() {
        let usdc = Address::from_str(ETHEREUM_USDC_TOKEN_ID).unwrap();
        let input = RoutedAsset::mock(Address::ZERO, Funding::Value, 1);
        let output = RoutedAsset::mock_permit2(ETHEREUM_USDC_TOKEN_ID);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(AssetId::from_chain(Chain::Ethereum), AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())))];
        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Ethereum).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(10_000_000_000_000_000u64),
            options: Options::default(),
        };

        let commands = build_commands(&request, &input, &output, 10_000_000_000_000_000, 25_000_000, &routes, None, true, UniversalRouterAbi::V2).unwrap();

        let UniversalRouterCommand::TRANSFER(fee) = &commands[0] else {
            panic!("expected TRANSFER fee from the router balance");
        };
        assert_eq!(fee.token, Address::ZERO);
        let UniversalRouterCommand::V4_SWAP { actions } = &commands[1] else {
            panic!("expected V4_SWAP");
        };
        assert!(matches!(&actions[1], SETTLE { currency, payer_is_user: false, .. } if *currency == Address::ZERO));
        assert!(matches!(&actions[2], TAKE { currency, .. } if *currency == usdc));
    }

    #[test]
    fn test_build_commands_celo_tokenized_native() {
        let celo = RoutedAsset::mock_permit2(CELO_WETH_TOKEN_ID);
        let usdt = RoutedAsset::mock_permit2(CELO_USDT_TOKEN_ID);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())), AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())))];

        // CELO -> USDT: no wrap, direct swap through token path
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::parse_bytes(b"22000000000000000000", 10).unwrap(),
            options: Options::default(),
        };
        let commands = build_commands(&request, &celo, &usdt, 22_000_000_000_000_000_000, 14_804_757, &routes, None, false, UniversalRouterAbi::V2).unwrap();

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
            value: BigUint::from(900000u64),
            options: Options { slippage: 50.into(), use_max_amount: false },
        };
        let routes = vec![Route::mock(AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())), AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())))];
        let commands = build_commands(&request, &usdt, &celo, 900_000, 10_752_991_111_111_111_170, &routes, None, false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], UniversalRouterCommand::V4_SWAP { .. }));
        assert!(matches!(commands[1], UniversalRouterCommand::PAY_PORTION(_)));
        assert!(matches!(commands[2], UniversalRouterCommand::SWEEP(_)));
    }

    #[test]
    fn test_build_commands_v2_1_uses_v2_1_swap_action() {
        let celo = RoutedAsset::mock_permit2(CELO_WETH_TOKEN_ID);
        let usdt = RoutedAsset::mock_permit2(CELO_USDT_TOKEN_ID);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let routes = vec![Route::mock(AssetId::from(Chain::Celo, Some(CELO_WETH_TOKEN_ID.into())), AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())))];
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::parse_bytes(b"22000000000000000000", 10).unwrap(),
            options: Options::default(),
        };
        let commands = build_commands(&request, &celo, &usdt, 22_000_000_000_000_000_000, 14_804_757, &routes, None, false, UniversalRouterAbi::V2_1).unwrap();

        match &commands[0] {
            UniversalRouterCommand::V4_SWAP { actions } => assert!(matches!(actions[0], SWAP_EXACT_IN_V2_1(_))),
            _ => panic!("expected V4 swap command"),
        }
    }
}
