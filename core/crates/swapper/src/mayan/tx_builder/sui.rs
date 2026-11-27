use std::{
    collections::{BTreeSet, HashMap},
    fmt::{Debug, Display},
};

use gem_client::Client;
use gem_sui::tx_builder::{Argument, TransactionBuilder};
use gem_sui::{
    SUI_COIN_TYPE, SuiClient, coin_type_matches, is_sui_coin,
    models::{Coin, OwnedCoins},
    tx_builder::{ResolvedObjectInput, TransactionJsonReplay, build_input_coin, prepare_transaction_json_replay},
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    SwapperError,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        client::MayanClient,
        model::{GetSwapSuiParams, MayanQuoteCommon, SuiClientSwap},
        tx_builder::amount::fractional_amount,
    },
    route_cache::Cache,
};

pub(in crate::mayan::tx_builder) struct SuiClientSwapReplay {
    swap: SuiClientSwap,
    replay: TransactionJsonReplay,
}

impl SuiClientSwapReplay {
    pub(in crate::mayan::tx_builder) async fn get<C>(client: &MayanClient<C>, sui_client: &SuiClient, params: GetSwapSuiParams, wormhole_fee: u64) -> Result<Self, SwapperError>
    where
        C: Client + Clone + Send + Sync + Debug + 'static,
    {
        let coin_type = params.input_coin_type.clone();
        let amount = params.amount_in64.parse::<u64>()?;
        let swap: SuiClientSwap = client.get_swap_sui(params).await?;
        let replay = prepare_transaction_json_replay(sui_client, &swap.tx).await.map_err(sui_error)?;
        verify_wallet_spend(&replay, &coin_type, amount, wormhole_fee)?;
        Ok(Self { swap, replay })
    }
}

fn verify_wallet_spend(replay: &TransactionJsonReplay, coin_type: &str, amount: u64, wormhole_fee: u64) -> Result<(), SwapperError> {
    let spend = replay.wallet_spend().map_err(|_| SwapperError::InvalidRoute)?;
    let within_quote = spend.iter().all(|(spent_type, spent)| {
        let quoted = if coin_type_matches(spent_type, coin_type) { amount } else { 0 };
        let fee = if coin_type_matches(spent_type, SUI_COIN_TYPE) { wormhole_fee } else { 0 };
        *spent <= quoted.saturating_add(fee)
    });
    if !within_quote {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(())
}

pub(in crate::mayan::tx_builder) struct SuiInputCoin {
    pub txb: TransactionBuilder,
    pub coin: Argument,
    pub wh_fee_coin: WormholeFeeCoin,
}

#[derive(Debug, PartialEq)]
pub(in crate::mayan::tx_builder) enum WormholeFeeCoin {
    FromSwap(Argument),
    FromWallet { sui_address_balance: u64 },
}

impl WormholeFeeCoin {
    pub fn build_coin(self, txb: &mut TransactionBuilder, bridge_fee: u64) -> Result<Argument, SwapperError> {
        match self {
            Self::FromSwap(coin) => Ok(coin),
            Self::FromWallet { sui_address_balance } => build_input_coin(txb, SUI_COIN_TYPE, bridge_fee, &OwnedCoins::sui_address_balance(sui_address_balance)).map_err(sui_error),
        }
    }
}

pub(in crate::mayan::tx_builder) fn build_sui_input_coin(swap: Option<&SuiClientSwapReplay>, coin_type: &str, amount: u64, coins: &OwnedCoins<Coin>, sui_address_balance: u64) -> Result<SuiInputCoin, SwapperError> {
    if let Some(swap) = swap {
        let replayed = swap.replay.replay_with_address_balance(sui_address_balance).map_err(sui_error)?;
        let coin = replayed.argument(&swap.swap.out_coin).map_err(sui_error)?;
        let wh_fee_coin = match swap.swap.wh_fee_coin.as_ref() {
            Some(argument) => WormholeFeeCoin::FromSwap(replayed.argument(argument).map_err(sui_error)?),
            None => WormholeFeeCoin::FromWallet {
                sui_address_balance: replayed.address_balance,
            },
        };
        return Ok(SuiInputCoin { txb: replayed.txb, coin, wh_fee_coin });
    }

    let mut txb = TransactionBuilder::new();
    let coin = build_input_coin(&mut txb, coin_type, amount, coins).map_err(sui_error)?;
    Ok(SuiInputCoin {
        txb,
        coin,
        wh_fee_coin: WormholeFeeCoin::FromWallet { sui_address_balance },
    })
}

pub(in crate::mayan::tx_builder) fn requires_client_swap(route: &MayanQuoteCommon, input_contract: &str) -> bool {
    route.from_token.contract != input_contract
}

pub(in crate::mayan::tx_builder) fn bridge_amount(route: &MayanQuoteCommon, min_middle_amount: Option<&Value>, input_contract: &str) -> Result<u64, SwapperError> {
    if !requires_client_swap(route, input_contract) {
        return route.effective_amount_in64.parse::<u64>().map_err(SwapperError::from);
    }
    fractional_amount(min_middle_amount.ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

pub(in crate::mayan::tx_builder) async fn get_sui_input_coins(client: &SuiClient, sender: &str, route: &MayanQuoteCommon, input_contract: &str) -> Result<OwnedCoins<Coin>, SwapperError> {
    if requires_client_swap(route, input_contract) || is_sui_coin(input_contract) {
        return Ok(OwnedCoins::default());
    }
    client.get_coins(sender, input_contract).await.map_err(sui_error)
}

pub(in crate::mayan::tx_builder) async fn get_sui_objects(client: &SuiClient, object_ids: Vec<String>) -> Result<HashMap<String, ResolvedObjectInput>, SwapperError> {
    let object_ids: Vec<String> = object_ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
    let objects = ResolvedObjectInput::get_multiple(client, object_ids.clone()).await.map_err(sui_error)?;
    if objects.len() != object_ids.len() {
        return Err(SwapperError::transaction_error("Failed to load all Mayan Sui objects"));
    }
    Ok(object_ids.into_iter().zip(objects).collect())
}

pub(in crate::mayan::tx_builder) async fn get_package_id(client: &SuiClient, state_object_id: &'static str, package_id_cache: &Cache<&'static str, String>) -> Result<String, SwapperError> {
    if let Some(package_id) = package_id_cache.get(&state_object_id) {
        return Ok(package_id);
    }
    let state: MayanState = client.get_object_json(state_object_id.to_string()).await.map_err(sui_error)?;
    let package_id = state.latest_package_id;
    package_id_cache.put(state_object_id, package_id.clone());
    Ok(package_id)
}

pub(in crate::mayan::tx_builder) fn sui_error(err: impl Display) -> SwapperError {
    SwapperError::TransactionError(format!("Sui transaction error: {err}"))
}

#[derive(Debug, Deserialize)]
struct MayanState {
    latest_package_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mayan::model::{MayanFastMctpQuote, MayanMctpQuote, SuiTransactionArgument};
    use gem_sui::types::{Command, Input, WithdrawalReservation};
    use primitives::asset_constants::{SUI_USDC_TOKEN_ID, SUI_WAL_TOKEN_ID};

    #[test]
    fn test_bridge_amount() {
        let mut route = MayanMctpQuote::mock();
        let input_contract = route.mctp_input_contract.clone().unwrap();
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 1_000_000);

        route.common.from_token.contract = SUI_COIN_TYPE.to_string();
        route.min_middle_amount = Some(serde_json::json!(0.99));
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 990_000);

        let route = MayanFastMctpQuote::mock_sui();
        let input_contract = route.fast_mctp_input_contract.clone().unwrap();
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 1_038_528);
    }

    #[test]
    fn test_verify_wallet_spend() {
        let coin: SuiClientSwap = serde_json::from_str(include_str!("../test/sui_client_swap_sui_to_usdc.json")).unwrap();
        let token: SuiClientSwap = serde_json::from_str(include_str!("../test/sui_client_swap_wal_to_usdc.json")).unwrap();
        let wallet_coins = |coin_type: &'static str| (0..19).map(|index| (index, coin_type)).collect::<Vec<_>>();
        let from_gas = TransactionJsonReplay::mock(&coin.tx);
        let from_wallet_coins = TransactionJsonReplay::mock_with_wallet_coins(&token.tx, &wallet_coins(SUI_WAL_TOKEN_ID));

        assert_eq!(verify_wallet_spend(&from_gas, SUI_COIN_TYPE, 1_000_000_000, 0), Ok(()));
        assert_eq!(verify_wallet_spend(&from_wallet_coins, SUI_WAL_TOKEN_ID, 5_000_000_000, 0), Ok(()));
        for (replay, coin_type, amount, reason) in [
            (&from_gas, SUI_COIN_TYPE, 999_999_999, "a swap that takes more SUI than quoted"),
            (&from_gas, SUI_WAL_TOKEN_ID, 1_000_000_000, "a swap of a token that takes SUI"),
            (&from_wallet_coins, SUI_WAL_TOKEN_ID, 4_999_999_999, "a swap that takes more of the token than quoted"),
            (
                &TransactionJsonReplay::mock_with_wallet_coins(&token.tx, &wallet_coins(SUI_USDC_TOKEN_ID)),
                SUI_WAL_TOKEN_ID,
                5_000_000_000,
                "a swap that takes the wallet's coins of another token",
            ),
        ] {
            assert_eq!(verify_wallet_spend(replay, coin_type, amount, 0), Err(SwapperError::InvalidRoute), "{reason}");
        }
        assert_eq!(verify_wallet_spend(&from_gas, SUI_WAL_TOKEN_ID, 0, 1_000_000_000), Ok(()), "SUI for the Wormhole fee beside a token swap");
    }

    #[test]
    fn test_build_sui_input_coin() {
        let address_balance = 626_400_000_000;
        let swap: SuiClientSwap = serde_json::from_str(include_str!("../test/sui_client_swap_sui_to_usdc.json")).unwrap();
        let replay = SuiClientSwapReplay {
            replay: TransactionJsonReplay::mock(&swap.tx),
            swap: swap.clone(),
        };

        let funded = build_sui_input_coin(Some(&replay), SUI_USDC_TOKEN_ID, 1_000_000_000, &OwnedCoins::default(), address_balance).unwrap();
        assert_eq!(funded.coin, Argument::Result(6));
        assert_eq!(funded.wh_fee_coin, WormholeFeeCoin::FromWallet { sui_address_balance: 0 });

        let coins_only = build_sui_input_coin(Some(&replay), SUI_USDC_TOKEN_ID, 1_000_000_000, &OwnedCoins::default(), 0).unwrap();
        assert_eq!(coins_only.coin, Argument::Result(4));

        let with_wh_fee = SuiClientSwapReplay {
            replay: TransactionJsonReplay::mock(&swap.tx),
            swap: SuiClientSwap {
                wh_fee_coin: Some(SuiTransactionArgument::NestedResult { nested_result: [0, 0] }),
                ..swap
            },
        };
        let funded = build_sui_input_coin(Some(&with_wh_fee), SUI_USDC_TOKEN_ID, 1_000_000_000, &OwnedCoins::default(), address_balance).unwrap();
        assert_eq!(funded.wh_fee_coin, WormholeFeeCoin::FromSwap(Argument::NestedResult(2, 0)));

        let usdc_coins = OwnedCoins::new(SUI_USDC_TOKEN_ID.to_string(), vec![], 2_000_000);
        let without_swap = build_sui_input_coin(None, SUI_USDC_TOKEN_ID, 1_000_000, &usdc_coins, address_balance).unwrap();
        assert_eq!(without_swap.wh_fee_coin, WormholeFeeCoin::FromWallet { sui_address_balance: address_balance });
    }

    #[test]
    fn test_wormhole_fee_coin_build_coin() {
        let built = |fee_coin: WormholeFeeCoin, bridge_fee: u64| {
            let txb = &mut TransactionBuilder::new();
            let coin = fee_coin.build_coin(txb, bridge_fee).unwrap();
            let ptb = std::mem::take(txb).build().unwrap();
            let withdrawals = ptb
                .inputs
                .iter()
                .filter_map(|input| match input {
                    Input::FundsWithdrawal(withdrawal) => match withdrawal.reservation {
                        WithdrawalReservation::MaxAmount(amount) => Some(amount),
                    },
                    _ => None,
                })
                .collect::<Vec<_>>();
            let commands = ptb
                .commands
                .iter()
                .map(|command| match command {
                    Command::MoveCall(call) => format!("{}::{}", call.module, call.function),
                    Command::MergeCoins(merge) => format!("merge into {:?}", merge.coin),
                    Command::SplitCoins(split) => format!("split {:?}", split.coin),
                    command => format!("{command:?}"),
                })
                .collect::<Vec<_>>();
            (coin, withdrawals, commands)
        };

        assert_eq!(built(WormholeFeeCoin::FromSwap(Argument::Result(7)), 1_000_000), (Argument::Result(7), vec![], vec![]));
        assert_eq!(built(WormholeFeeCoin::FromWallet { sui_address_balance: 0 }, 1_000_000), (Argument::NestedResult(0, 0), vec![], vec!["split Gas".to_string()]));
        assert_eq!(
            built(WormholeFeeCoin::FromWallet { sui_address_balance: 5_000_000 }, 1_000_000),
            (Argument::Result(0), vec![1_000_000], vec!["coin::redeem_funds".to_string()])
        );
        assert_eq!(
            built(WormholeFeeCoin::FromWallet { sui_address_balance: 400_000 }, 1_000_000),
            (Argument::NestedResult(2, 0), vec![400_000], vec!["coin::redeem_funds".to_string(), "merge into Gas".to_string(), "split Gas".to_string()])
        );
        assert_eq!(built(WormholeFeeCoin::FromWallet { sui_address_balance: 5_000_000 }, 0), (Argument::Result(0), vec![], vec!["coin::zero".to_string()]));
    }
}
