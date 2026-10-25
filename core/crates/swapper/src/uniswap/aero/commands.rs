use crate::{
    SwapperError, eth_address,
    fees::{ReferralFee, bps_of},
    uniswap::routed_asset::{Funding, RoutedAsset},
};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{SolCall, SolValue};
use gem_evm::uniswap::{
    contracts::aero::{
        BALANCE_CHECK_COMMAND, BalanceCheck, BalanceSpend, CL_SWAP_EXACT_IN_COMMAND, FUND_ERC20_COMMAND, FundErc20, IMetarouter, SWEEP_COMMAND, SwapExactIn, Sweep, TRANSFER_COMMAND, Transfer, UNWRAP_WETH_COMMAND, UnwrapWeth,
        WRAP_ETH_COMMAND,
    },
    deployment::aero::METAROUTER,
};

type Command = (u8, Vec<u8>);

#[cfg_attr(test, derive(Clone))]
pub(super) struct SwapCommands {
    pub input: RoutedAsset,
    pub output: RoutedAsset,
    pub pools: Vec<Address>,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub recipient: Address,
    pub referral: ReferralFee,
    pub fee_token_is_input: bool,
}

pub(super) fn build_commands(swap: &SwapCommands) -> Result<Vec<Command>, SwapperError> {
    let (fee_recipient, metarouter) = (eth_address::parse_str(&swap.referral.address)?, eth_address::parse_str(METAROUTER)?);
    let fee = if swap.fee_token_is_input { bps_of(&swap.amount_in, swap.referral.bps) } else { U256::ZERO };
    let output_fee_bps = (!swap.fee_token_is_input && swap.referral.bps > 0).then_some(swap.referral.bps);
    let unwrap_output = swap.output.funding == Funding::Value;
    let swap_to_router = output_fee_bps.is_some() || unwrap_output;
    let payer_is_user = swap.input.funding == Funding::Token && fee.is_zero();

    let funding = match swap.input.funding {
        Funding::Value => Some(command(WRAP_ETH_COMMAND, BalanceSpend::amount(swap.amount_in))),
        Funding::Token if !payer_is_user => Some(command(
            FUND_ERC20_COMMAND,
            FundErc20 {
                token: swap.input.address,
                spend: BalanceSpend::amount(swap.amount_in),
            },
        )),
        Funding::Token | Funding::RouterBalance => None,
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
            recipient: if swap_to_router { metarouter } else { swap.recipient },
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
                    owner: metarouter,
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
        asset_constants::{ARC_EURC_TOKEN_ID, ARC_USDC_TOKEN_ID, BASE_CBBTC_TOKEN_ID, BASE_USDC_TOKEN_ID},
        swap::EVM_REFERRAL_ADDRESS,
    };

    const TEST_POOL: Address = Address::repeat_byte(0x11);
    const TEST_RECIPIENT: Address = Address::repeat_byte(0x22);

    fn exact_in(token_in: Address, amount_in: u64, payer_is_user: bool, min_amount_out: u64, recipient: Address) -> Command {
        let swap = SwapExactIn {
            pools: vec![TEST_POOL],
            tokenIn: token_in,
            amountIn: BalanceSpend::amount(U256::from(amount_in)),
            payerIsUser: payer_is_user,
            minAmountOut: U256::from(min_amount_out),
            recipient,
        };
        command(CL_SWAP_EXACT_IN_COMMAND, swap)
    }

    #[test]
    fn test_build_commands() {
        let (usdc, cbbtc) = (RoutedAsset::mock_token(BASE_USDC_TOKEN_ID), RoutedAsset::mock_token(BASE_CBBTC_TOKEN_ID));
        let (arc_usdc, eurc) = (RoutedAsset::mock_router_balance(ARC_USDC_TOKEN_ID, 1_000_000_000_000), RoutedAsset::mock_token(ARC_EURC_TOKEN_ID));
        let (referral, metarouter): (Address, Address) = (EVM_REFERRAL_ADDRESS.parse().unwrap(), METAROUTER.parse().unwrap());
        let (fee, share) = (
            BalanceSpend::amount(U256::from(500_000u64)),
            BalanceSpend {
                mode: SpendMode::Pips,
                value: U256::from(5_000),
            },
        );
        let usdc_to_cbbtc = SwapCommands {
            input: usdc.clone(),
            output: cbbtc.clone(),
            pools: vec![TEST_POOL],
            amount_in: U256::from(100_000_000u64),
            min_amount_out: U256::from(120_000u64),
            recipient: TEST_RECIPIENT,
            referral: default_referral_fees().evm,
            fee_token_is_input: true,
        };
        let cases = [
            (
                usdc_to_cbbtc.clone(),
                vec![
                    command(FUND_ERC20_COMMAND, (usdc.address, BalanceSpend::amount(U256::from(100_000_000u64)))),
                    command(TRANSFER_COMMAND, (usdc.address, referral, fee.clone())),
                    exact_in(usdc.address, 99_500_000, false, 120_000, TEST_RECIPIENT),
                ],
            ),
            (
                SwapCommands {
                    input: cbbtc.clone(),
                    output: usdc.clone(),
                    fee_token_is_input: false,
                    ..usdc_to_cbbtc.clone()
                },
                vec![
                    exact_in(cbbtc.address, 100_000_000, true, 0, metarouter),
                    command(TRANSFER_COMMAND, (usdc.address, referral, share)),
                    command(SWEEP_COMMAND, (usdc.address, TEST_RECIPIENT, U256::from(120_000u64))),
                ],
            ),
            (
                SwapCommands {
                    referral: ReferralFee { bps: 0, ..default_referral_fees().evm },
                    ..usdc_to_cbbtc.clone()
                },
                vec![exact_in(usdc.address, 100_000_000, true, 120_000, TEST_RECIPIENT)],
            ),
            (
                SwapCommands {
                    input: arc_usdc.clone(),
                    output: eurc,
                    ..usdc_to_cbbtc.clone()
                },
                vec![command(TRANSFER_COMMAND, (arc_usdc.address, referral, fee)), exact_in(arc_usdc.address, 99_500_000, false, 120_000, TEST_RECIPIENT)],
            ),
        ];

        for (swap, expected) in cases {
            assert_eq!(build_commands(&swap).unwrap(), expected);
        }
    }
}
