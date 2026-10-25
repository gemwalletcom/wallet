use crate::{
    SwapperError, eth_address,
    fees::{ReferralFee, bps_of},
    uniswap::routed_asset::{Funding, RoutedAsset},
};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{SolCall, SolValue};
use gem_evm::uniswap::contracts::aero::{
    BALANCE_CHECK_COMMAND, BalanceCheck, BalanceSpend, CL_SWAP_EXACT_IN_COMMAND, FUND_ERC20_COMMAND, FundErc20, IMetarouter, SWEEP_COMMAND, SwapExactIn, Sweep, TRANSFER_COMMAND, Transfer, UNWRAP_WETH_COMMAND, UnwrapWeth, WRAP_ETH_COMMAND,
};

type Command = (u8, Vec<u8>);

pub(super) struct SwapCommands {
    pub input: RoutedAsset,
    pub output: RoutedAsset,
    pub pools: Vec<Address>,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub recipient: Address,
    pub metarouter: Address,
    pub referral: ReferralFee,
    pub fee_token_is_input: bool,
}

pub(super) fn input_fee(amount_in: U256, bps: u32, fee_token_is_input: bool) -> U256 {
    match fee_token_is_input {
        true => bps_of(&amount_in, bps),
        false => U256::ZERO,
    }
}

pub(super) fn build_commands(swap: &SwapCommands) -> Result<Vec<Command>, SwapperError> {
    let fee_recipient = eth_address::parse_str(&swap.referral.address)?;
    let fee = input_fee(swap.amount_in, swap.referral.bps, swap.fee_token_is_input);
    let output_fee_bps = (!swap.fee_token_is_input && swap.referral.bps > 0).then_some(swap.referral.bps);
    let unwrap_output = swap.output.funding == Funding::Value;
    let swap_to_router = output_fee_bps.is_some() || unwrap_output;
    let payer_is_user = swap.input.funding == Funding::Permit2 && fee.is_zero();

    let funding = match swap.input.funding {
        Funding::Value => Some(command(WRAP_ETH_COMMAND, BalanceSpend::amount(swap.amount_in))),
        Funding::Permit2 if !payer_is_user => Some(command(
            FUND_ERC20_COMMAND,
            FundErc20 {
                token: swap.input.address,
                spend: BalanceSpend::amount(swap.amount_in),
            },
        )),
        Funding::Permit2 | Funding::RouterBalance => None,
    };
    let input_fee = (!fee.is_zero()).then(|| {
        command(
            TRANSFER_COMMAND,
            Transfer {
                token: swap.input.address,
                recipient: fee_recipient,
                spend: BalanceSpend::amount(fee),
            },
        )
    });
    let exact_in = command(
        CL_SWAP_EXACT_IN_COMMAND,
        SwapExactIn {
            pools: swap.pools.clone(),
            tokenIn: swap.input.address,
            amountIn: BalanceSpend::amount(swap.amount_in - fee),
            payerIsUser: payer_is_user,
            minAmountOut: if swap_to_router { U256::ZERO } else { swap.min_amount_out },
            recipient: if swap_to_router { swap.metarouter } else { swap.recipient },
        },
    );
    let output_fee = output_fee_bps.map(|bps| {
        command(
            TRANSFER_COMMAND,
            Transfer {
                token: swap.output.address,
                recipient: fee_recipient,
                spend: BalanceSpend::share(bps),
            },
        )
    });
    let settlement = match (unwrap_output, swap_to_router) {
        (true, _) => vec![
            command(
                BALANCE_CHECK_COMMAND,
                BalanceCheck {
                    asset: swap.output.address,
                    owner: swap.metarouter,
                    minBalance: swap.min_amount_out,
                },
            ),
            command(
                UNWRAP_WETH_COMMAND,
                UnwrapWeth {
                    recipient: swap.recipient,
                    spend: BalanceSpend::all(),
                },
            ),
        ],
        (false, true) => vec![command(
            SWEEP_COMMAND,
            Sweep {
                token: swap.output.address,
                recipient: swap.recipient,
                minAmount: swap.min_amount_out,
            },
        )],
        (false, false) => vec![],
    };
    Ok(funding.into_iter().chain(input_fee).chain([exact_in]).chain(output_fee).chain(settlement).collect())
}

pub(super) fn encode_execute(commands: Vec<Command>, deadline: U256) -> Vec<u8> {
    let (ids, inputs): (Vec<u8>, Vec<Bytes>) = commands.into_iter().map(|(id, input)| (id, Bytes::from(input))).unzip();
    IMetarouter::executeCall {
        commands: Bytes::from(ids),
        inputs,
        deadline,
    }
    .abi_encode()
}

fn command(id: u8, input: impl SolValue) -> Command {
    (id, input.abi_encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::default_referral_fees;
    use gem_evm::uniswap::contracts::aero::SpendMode;
    use primitives::{
        asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, BASE_CBBTC_TOKEN_ID, BASE_USDC_TOKEN_ID, BASE_WETH_TOKEN_ID},
        swap::EVM_REFERRAL_ADDRESS,
    };

    #[test]
    fn test_build_commands() {
        let (usdc, cbbtc, eth) = (RoutedAsset::mock_permit2(BASE_USDC_TOKEN_ID), RoutedAsset::mock_permit2(BASE_CBBTC_TOKEN_ID), RoutedAsset::mock_value(BASE_WETH_TOKEN_ID));
        let (arc_usdc, eurc) = (RoutedAsset::mock_router_balance(ARC_USDC_TOKEN_ID, 1_000_000_000_000), RoutedAsset::mock_permit2(ARC_EURC_TOKEN_ID));
        let (pool, recipient, metarouter) = (Address::repeat_byte(0x11), Address::repeat_byte(0x22), Address::repeat_byte(0x33));
        let referral = eth_address::parse_str(EVM_REFERRAL_ADDRESS).unwrap();
        let (amount_in, min_amount_out) = (U256::from(100_000_000u64), U256::from(120_000u64));
        let fee_share = BalanceSpend {
            mode: SpendMode::Pips,
            value: U256::from(5_000),
        };
        let usdc_to_cbbtc = SwapCommands {
            input: usdc.clone(),
            output: cbbtc.clone(),
            pools: vec![pool],
            amount_in,
            min_amount_out,
            recipient,
            metarouter,
            referral: default_referral_fees().evm,
            fee_token_is_input: true,
        };
        let cases = [
            (
                SwapCommands {
                    input: usdc.clone(),
                    output: cbbtc.clone(),
                    pools: vec![pool],
                    referral: default_referral_fees().evm,
                    ..usdc_to_cbbtc
                },
                vec![
                    command(
                        FUND_ERC20_COMMAND,
                        FundErc20 {
                            token: usdc.address,
                            spend: BalanceSpend::amount(amount_in),
                        },
                    ),
                    command(
                        TRANSFER_COMMAND,
                        Transfer {
                            token: usdc.address,
                            recipient: referral,
                            spend: BalanceSpend::amount(U256::from(500_000u64)),
                        },
                    ),
                    command(
                        CL_SWAP_EXACT_IN_COMMAND,
                        SwapExactIn {
                            pools: vec![pool],
                            tokenIn: usdc.address,
                            amountIn: BalanceSpend::amount(U256::from(99_500_000u64)),
                            payerIsUser: false,
                            minAmountOut: min_amount_out,
                            recipient,
                        },
                    ),
                ],
            ),
            (
                SwapCommands {
                    input: cbbtc.clone(),
                    output: usdc.clone(),
                    pools: vec![pool],
                    referral: default_referral_fees().evm,
                    fee_token_is_input: false,
                    ..usdc_to_cbbtc
                },
                vec![
                    command(
                        CL_SWAP_EXACT_IN_COMMAND,
                        SwapExactIn {
                            pools: vec![pool],
                            tokenIn: cbbtc.address,
                            amountIn: BalanceSpend::amount(amount_in),
                            payerIsUser: true,
                            minAmountOut: U256::ZERO,
                            recipient: metarouter,
                        },
                    ),
                    command(
                        TRANSFER_COMMAND,
                        Transfer {
                            token: usdc.address,
                            recipient: referral,
                            spend: fee_share.clone(),
                        },
                    ),
                    command(
                        SWEEP_COMMAND,
                        Sweep {
                            token: usdc.address,
                            recipient,
                            minAmount: min_amount_out,
                        },
                    ),
                ],
            ),
            (
                SwapCommands {
                    input: usdc.clone(),
                    output: cbbtc.clone(),
                    pools: vec![pool],
                    referral: ReferralFee { bps: 0, ..default_referral_fees().evm },
                    ..usdc_to_cbbtc
                },
                vec![command(
                    CL_SWAP_EXACT_IN_COMMAND,
                    SwapExactIn {
                        pools: vec![pool],
                        tokenIn: usdc.address,
                        amountIn: BalanceSpend::amount(amount_in),
                        payerIsUser: true,
                        minAmountOut: min_amount_out,
                        recipient,
                    },
                )],
            ),
            (
                SwapCommands {
                    input: cbbtc.clone(),
                    output: eth.clone(),
                    pools: vec![pool],
                    referral: ReferralFee { bps: 0, ..default_referral_fees().evm },
                    fee_token_is_input: false,
                    ..usdc_to_cbbtc
                },
                vec![
                    command(
                        CL_SWAP_EXACT_IN_COMMAND,
                        SwapExactIn {
                            pools: vec![pool],
                            tokenIn: cbbtc.address,
                            amountIn: BalanceSpend::amount(amount_in),
                            payerIsUser: true,
                            minAmountOut: U256::ZERO,
                            recipient: metarouter,
                        },
                    ),
                    command(
                        BALANCE_CHECK_COMMAND,
                        BalanceCheck {
                            asset: eth.address,
                            owner: metarouter,
                            minBalance: min_amount_out,
                        },
                    ),
                    command(UNWRAP_WETH_COMMAND, UnwrapWeth { recipient, spend: BalanceSpend::all() }),
                ],
            ),
            (
                SwapCommands {
                    input: arc_usdc.clone(),
                    output: eurc.clone(),
                    pools: vec![pool],
                    referral: default_referral_fees().evm,
                    ..usdc_to_cbbtc
                },
                vec![
                    command(
                        TRANSFER_COMMAND,
                        Transfer {
                            token: arc_usdc.address,
                            recipient: referral,
                            spend: BalanceSpend::amount(U256::from(500_000u64)),
                        },
                    ),
                    command(
                        CL_SWAP_EXACT_IN_COMMAND,
                        SwapExactIn {
                            pools: vec![pool],
                            tokenIn: arc_usdc.address,
                            amountIn: BalanceSpend::amount(U256::from(99_500_000u64)),
                            payerIsUser: false,
                            minAmountOut: min_amount_out,
                            recipient,
                        },
                    ),
                ],
            ),
            (
                SwapCommands {
                    input: eurc.clone(),
                    output: arc_usdc.clone(),
                    pools: vec![pool],
                    referral: default_referral_fees().evm,
                    fee_token_is_input: false,
                    ..usdc_to_cbbtc
                },
                vec![
                    command(
                        CL_SWAP_EXACT_IN_COMMAND,
                        SwapExactIn {
                            pools: vec![pool],
                            tokenIn: eurc.address,
                            amountIn: BalanceSpend::amount(amount_in),
                            payerIsUser: true,
                            minAmountOut: U256::ZERO,
                            recipient: metarouter,
                        },
                    ),
                    command(
                        TRANSFER_COMMAND,
                        Transfer {
                            token: arc_usdc.address,
                            recipient: referral,
                            spend: fee_share,
                        },
                    ),
                    command(
                        SWEEP_COMMAND,
                        Sweep {
                            token: arc_usdc.address,
                            recipient,
                            minAmount: min_amount_out,
                        },
                    ),
                ],
            ),
        ];

        for (swap, expected) in cases {
            assert_eq!(build_commands(&swap).unwrap(), expected);
        }
    }
}
