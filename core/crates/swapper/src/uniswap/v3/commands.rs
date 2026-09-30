use crate::{
    SwapperError, eth_address,
    fees::default_referral_fees,
    models::*,
    uniswap::routed_asset::{Funding, RoutedAsset},
};
use gem_evm::uniswap::{
    command::{ADDRESS_THIS, PayPortion, Permit2Permit, Sweep, Transfer, UniversalRouterCommand, UnwrapWeth, V3SwapExactIn, V3SwapExactInV2_1, WrapEth},
    deployment::UniversalRouterAbi,
};

use alloy_primitives::{Address, Bytes, U256};

pub fn build_commands(
    request: &QuoteRequest,
    input: &RoutedAsset,
    output: &RoutedAsset,
    amount_in: U256,
    quote_amount: U256,
    path: &Bytes,
    permit: Option<Permit2Permit>,
    fee_token_is_input: bool,
    universal_router_abi: UniversalRouterAbi,
) -> Result<Vec<UniversalRouterCommand>, SwapperError> {
    let fee_options = default_referral_fees().evm;
    let recipient = eth_address::parse_str(&request.wallet_address)?;

    let token_in = input.address;
    let token_out = output.address;
    let payer_is_user = input.funding == Funding::Permit2;
    let unwrap_output_weth = output.funding == Funding::Value;
    let pay_fees = fee_options.bps > 0;

    let mut commands: Vec<UniversalRouterCommand> = vec![];

    let amount_out = quote_amount;
    match input.funding {
        Funding::Value => commands.push(UniversalRouterCommand::WRAP_ETH(WrapEth {
            recipient: eth_address::parse_str(ADDRESS_THIS)?,
            amount_min: amount_in,
        })),
        Funding::RouterBalance => {}
        Funding::Permit2 => {
            if let Some(permit) = permit {
                commands.push(UniversalRouterCommand::PERMIT2_PERMIT(permit));
            }
        }
    }

    if pay_fees {
        if fee_token_is_input {
            let fee = amount_in * U256::from(fee_options.bps) / U256::from(10000);
            let fee_transfer = Transfer {
                token: token_in,
                recipient: eth_address::parse_str(&fee_options.address)?,
                value: fee,
            };
            commands.push(match input.funding {
                Funding::Value | Funding::RouterBalance => UniversalRouterCommand::TRANSFER(fee_transfer),
                Funding::Permit2 => UniversalRouterCommand::PERMIT2_TRANSFER_FROM(fee_transfer),
            });

            commands.push(build_v3_swap_exact_in_command(recipient, amount_in - fee, amount_out, path.clone(), payer_is_user, universal_router_abi));
        } else {
            commands.push(build_v3_swap_exact_in_command(
                eth_address::parse_str(ADDRESS_THIS)?,
                amount_in,
                U256::from(0),
                path.clone(),
                payer_is_user,
                universal_router_abi,
            ));

            commands.push(UniversalRouterCommand::PAY_PORTION(PayPortion {
                token: token_out,
                recipient: eth_address::parse_str(&fee_options.address)?,
                bips: U256::from(fee_options.bps),
            }));

            if !unwrap_output_weth {
                commands.push(UniversalRouterCommand::SWEEP(Sweep {
                    token: token_out,
                    recipient,
                    amount_min: U256::from(amount_out),
                }));
            }
        }
    } else {
        commands.push(build_v3_swap_exact_in_command(recipient, amount_in, amount_out, path.clone(), payer_is_user, universal_router_abi));
    }

    if unwrap_output_weth {
        commands.push(UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
            recipient,
            amount_min: U256::from(amount_out),
        }));
    }
    Ok(commands)
}

fn build_v3_swap_exact_in_command(recipient: Address, amount_in: U256, amount_out_min: U256, path: Bytes, payer_is_user: bool, universal_router_abi: UniversalRouterAbi) -> UniversalRouterCommand {
    match universal_router_abi {
        UniversalRouterAbi::V2 => UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
            recipient,
            amount_in,
            amount_out_min,
            path,
            payer_is_user,
        }),
        UniversalRouterAbi::V2_1 => UniversalRouterCommand::V3_SWAP_EXACT_IN_V2_1(V3SwapExactInV2_1 {
            recipient,
            amount_in,
            amount_out_min,
            path,
            payer_is_user,
            min_hop_price_x36: vec![],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permit2_data::*;
    use crate::uniswap::routed_asset::RoutedAsset;
    use alloy_primitives::{address, aliases::U256};
    use gem_evm::uniswap::{FeeTier, path::build_direct_pair};
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, CELO_USDT_TOKEN_ID, CELO_WETH_TOKEN_ID, ROBINHOOD_USDG_TOKEN_ID, ROBINHOOD_WETH_TOKEN_ID},
        asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID, OPTIMISM_USDC_E_TOKEN_ID, OPTIMISM_USDC_TOKEN_ID, OPTIMISM_USDT_TOKEN_ID, OPTIMISM_WETH_TOKEN_ID},
        contract_constants::OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT,
        swap::EVM_REFERRAL_ADDRESS,
    };
    use std::str::FromStr;

    const TEST_WALLET: Address = address!("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
    const TEST_ROUTER: Address = address!("0x0000000000000000000000000000000000000002");

    #[test]
    fn test_build_commands_router_balance_input() {
        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Arc).into(),
            to_asset: AssetId::from(Chain::Arc, Some(ARC_EURC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(3_000_000_000_000_000_000u64),
            options: Options::default(),
        };
        let input = RoutedAsset::mock_router_balance(ARC_USDC_TOKEN_ID, 1_000_000_000_000);
        let output = RoutedAsset::mock_permit2(ARC_EURC_TOKEN_ID);
        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let amount_in = U256::from(3_000_000u64);
        let fee = amount_in * U256::from(default_referral_fees().evm.bps) / U256::from(10000);

        let commands = super::build_commands(&request, &input, &output, amount_in, U256::from(2_523_162u64), &path, None, true, UniversalRouterAbi::V2_1).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::TRANSFER(Transfer {
                    token: input.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    value: fee,
                }),
                UniversalRouterCommand::V3_SWAP_EXACT_IN_V2_1(V3SwapExactInV2_1 {
                    recipient: TEST_WALLET,
                    amount_in: amount_in - fee,
                    amount_out_min: U256::from(2_523_162u64),
                    path,
                    payer_is_user: false,
                    min_hop_price_x36: vec![],
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_eth_to_token() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Ethereum, None).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(10000000000000000u64),
            options: Options::default(),
        };

        let input = RoutedAsset::mock_value(ETHEREUM_WETH_TOKEN_ID);
        let output = RoutedAsset::mock_permit2(ETHEREUM_USDC_TOKEN_ID);
        let amount_in = U256::from(1000000000000000u64);

        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(&request, &input, &output, amount_in, U256::from(0), &path, None, false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::WRAP_ETH(WrapEth {
                    recipient: TEST_ROUTER,
                    amount_min: amount_in,
                }),
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: false,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: output.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::SWEEP(Sweep {
                    token: output.address,
                    recipient: TEST_WALLET,
                    amount_min: U256::ZERO,
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_v2_1_uses_v2_1_swap_payload() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Ethereum, None).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(10000000000000000u64),
            options: Options::default(),
        };

        let input = RoutedAsset::mock_value(ETHEREUM_WETH_TOKEN_ID);
        let output = RoutedAsset::mock_permit2(ETHEREUM_USDC_TOKEN_ID);
        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(&request, &input, &output, U256::from(1000000000000000u64), U256::from(0), &path, None, false, UniversalRouterAbi::V2_1).unwrap();

        assert_eq!(
            commands[1],
            UniversalRouterCommand::V3_SWAP_EXACT_IN_V2_1(V3SwapExactInV2_1 {
                recipient: TEST_ROUTER,
                amount_in: U256::from(1000000000000000u64),
                amount_out_min: U256::ZERO,
                path,
                payer_is_user: false,
                min_hop_price_x36: vec![],
            })
        );
    }

    #[test]
    fn test_build_commands_v2_1_usdg_to_eth() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Robinhood, Some(ROBINHOOD_USDG_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Robinhood, None).into(),
            wallet_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            destination_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            value: BigUint::from(10000u64),
            options: Options::default(),
        };

        let deployment = gem_evm::uniswap::deployment::v3::get_uniswap_router_deployment_by_chain(&Chain::Robinhood).unwrap();
        let input = RoutedAsset::mock_permit2(ROBINHOOD_USDG_TOKEN_ID);
        let output = RoutedAsset::mock_value(ROBINHOOD_WETH_TOKEN_ID);
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();
        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: ROBINHOOD_USDG_TOKEN_ID.into(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1782952655,
                    nonce: 0,
                },
                spender: deployment.universal_router.into(),
                sig_deadline: 1782952655,
            },
            signature: vec![0; 65],
        };

        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(
            &request,
            &input,
            &output,
            amount_in,
            U256::from(250_000_000_000u64),
            &path,
            Some(permit2_data.clone().try_into().unwrap()),
            false,
            deployment.universal_router_abi,
        )
        .unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::PERMIT2_PERMIT(permit2_data.try_into().unwrap()),
                UniversalRouterCommand::V3_SWAP_EXACT_IN_V2_1(V3SwapExactInV2_1 {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: true,
                    min_hop_price_x36: vec![],
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: output.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
                    recipient: address!("0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"),
                    amount_min: U256::from(250_000_000_000u64),
                }),
            ]
        );
        assert_eq!(commands[1].encode().len(), 320);
    }

    #[test]
    fn test_build_commands_usdc_to_usdt() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDT_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(6500000u64),
            options: Options::default(),
        };

        let input = RoutedAsset::mock_permit2(request.from_asset.asset_id().token_id.as_ref().unwrap());
        let output = RoutedAsset::mock_permit2(request.to_asset.asset_id().token_id.as_ref().unwrap());
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: OPTIMISM_USDC_TOKEN_ID.into(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667593,
                    nonce: 0,
                },
                spender: OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.into(),
                sig_deadline: 1730077393,
            },
            signature: hex::decode("8f32d2e66506a4f424b1b23309ed75d338534d0912129a8aa3381fab4eb8032f160e0988f10f512b19a58c2a689416366c61cc0c483c3b5322dc91f8b60107671b").unwrap(),
        };

        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(&request, &input, &output, amount_in, U256::from(6507936), &path, Some(permit2_data.clone().try_into().unwrap()), false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::PERMIT2_PERMIT(permit2_data.try_into().unwrap()),
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: true,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: output.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::SWEEP(Sweep {
                    token: output.address,
                    recipient: TEST_WALLET,
                    amount_min: U256::from(6507936u64),
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_usdc_to_aave() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, Some("0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into())).into(),
            to_asset: AssetId::from(Chain::Optimism, Some("0x76fb31fb4af56892a25e32cfc43de717950c9278".into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(5064985u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
        };

        let input = RoutedAsset::mock_permit2(request.from_asset.asset_id().token_id.as_ref().unwrap());
        let output = RoutedAsset::mock_permit2(request.to_asset.asset_id().token_id.as_ref().unwrap());
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();
        let quote_amount = U256::from(33377662359182269u64);
        let fee = amount_in * U256::from(default_referral_fees().evm.bps) / U256::from(10000);

        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(&request, &input, &output, amount_in, quote_amount, &path, None, false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path: path.clone(),
                    payer_is_user: true,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: output.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::SWEEP(Sweep {
                    token: output.address,
                    recipient: TEST_WALLET,
                    amount_min: quote_amount,
                }),
            ]
        );

        let commands = super::build_commands(&request, &input, &output, amount_in, quote_amount, &path, None, true, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::PERMIT2_TRANSFER_FROM(Transfer {
                    token: input.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    value: fee,
                }),
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_WALLET,
                    amount_in: amount_in - fee,
                    amount_out_min: quote_amount,
                    path,
                    payer_is_user: true,
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_usdce_to_eth() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, Some(OPTIMISM_USDC_E_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Ethereum, None).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(10000000u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
        };

        let input = RoutedAsset::mock_permit2(request.from_asset.asset_id().token_id.as_ref().unwrap());
        let output = RoutedAsset::mock_value(OPTIMISM_WETH_TOKEN_ID);
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();

        let permit2_data = Permit2Data {
            permit_single: PermitSingle {
                details: Permit2Detail {
                    token: request.from_asset.asset_id().token_id.unwrap(),
                    amount: "1461501637330902918203684832716283019655932542975".into(),
                    expiration: 1732667502,
                    nonce: 0,
                },
                spender: OPTIMISM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT.into(),
                sig_deadline: 1730077302,
            },
            signature: hex::decode("00e96ed0f5bf5cca62dc9d9753960d83c8be83224456559a1e93a66d972a019f6f328a470f8257d3950b4cb7cd0024d789b4fcd9e80c4eb43d82a38d9e5332f31b").unwrap(),
        };

        let path = build_direct_pair(&input.address, &output.address, FeeTier::FiveHundred);
        let commands = super::build_commands(
            &request,
            &input,
            &output,
            amount_in,
            U256::from(3997001989341576u64),
            &path,
            Some(permit2_data.clone().try_into().unwrap()),
            false,
            UniversalRouterAbi::V2,
        )
        .unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::PERMIT2_PERMIT(permit2_data.try_into().unwrap()),
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: true,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: output.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::UNWRAP_WETH(UnwrapWeth {
                    recipient: TEST_WALLET,
                    amount_min: U256::from(3997001989341576u64),
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_eth_to_uni_with_input_fee() {
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Optimism, None).into(),
            to_asset: AssetId::from(Chain::Optimism, Some("0x6fd9d7ad17242c41f7131d257212c54a0e816691".into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(1000000000000000u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
        };

        let input = RoutedAsset::mock_value(OPTIMISM_WETH_TOKEN_ID);
        let output = RoutedAsset::mock_permit2(&request.to_asset.asset_id().token_id.unwrap());
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();
        let quote_amount = U256::from(244440440678888410_u64);
        let fee = amount_in * U256::from(default_referral_fees().evm.bps) / U256::from(10000);

        let path = build_direct_pair(&input.address, &output.address, FeeTier::ThreeThousand);
        let commands = super::build_commands(&request, &input, &output, amount_in, quote_amount, &path, None, true, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::WRAP_ETH(WrapEth {
                    recipient: TEST_ROUTER,
                    amount_min: amount_in,
                }),
                UniversalRouterCommand::TRANSFER(Transfer {
                    token: input.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    value: fee,
                }),
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_WALLET,
                    amount_in: amount_in - fee,
                    amount_out_min: quote_amount,
                    path,
                    payer_is_user: false,
                }),
            ]
        );
    }

    #[test]
    fn test_build_commands_celo_tokenized_native() {
        let celo = RoutedAsset::mock_permit2(CELO_WETH_TOKEN_ID);
        let usdt = RoutedAsset::mock_permit2(CELO_USDT_TOKEN_ID);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, None).into(),
            to_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::parse_bytes(b"22000000000000000000", 10).unwrap(),
            options: Options::default(),
        };
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();
        let path = build_direct_pair(&celo.address, &usdt.address, FeeTier::Hundred);
        let commands = super::build_commands(&request, &celo, &usdt, amount_in, U256::from(14804757u64), &path, None, false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: true,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: usdt.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::SWEEP(Sweep {
                    token: usdt.address,
                    recipient: TEST_WALLET,
                    amount_min: U256::from(14804757u64),
                }),
            ]
        );

        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::Celo, Some(CELO_USDT_TOKEN_ID.into())).into(),
            to_asset: AssetId::from(Chain::Celo, None).into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(900000u64),
            options: Options { slippage: 50.into(), use_max_amount: false },
        };
        let amount_in = U256::from_str(&request.value.to_string()).unwrap();
        let path = build_direct_pair(&usdt.address, &celo.address, FeeTier::Hundred);
        let commands = super::build_commands(&request, &usdt, &celo, amount_in, U256::from(10752991111111111170u128), &path, None, false, UniversalRouterAbi::V2).unwrap();

        assert_eq!(
            commands,
            vec![
                UniversalRouterCommand::V3_SWAP_EXACT_IN(V3SwapExactIn {
                    recipient: TEST_ROUTER,
                    amount_in,
                    amount_out_min: U256::ZERO,
                    path,
                    payer_is_user: true,
                }),
                UniversalRouterCommand::PAY_PORTION(PayPortion {
                    token: celo.address,
                    recipient: eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap(),
                    bips: U256::from(default_referral_fees().evm.bps),
                }),
                UniversalRouterCommand::SWEEP(Sweep {
                    token: celo.address,
                    recipient: TEST_WALLET,
                    amount_min: U256::from(10752991111111111170u128),
                }),
            ]
        );
    }
}
