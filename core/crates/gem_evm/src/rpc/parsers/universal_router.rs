use std::str::FromStr;

use alloy_primitives::{Address, B256, U256, keccak256};
use alloy_sol_types::{SolCall, SolEvent, SolValue};

use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    rpc::{mapper::TRANSFER_TOPIC, model::TransactionReceipt},
    u256::u256_to_biguint,
    uniswap::{
        actions::{V4Action, decode_action_data},
        command::{
            MSG_SENDER, SWEEP_COMMAND, Sweep, UNWRAP_WETH_COMMAND, UnwrapWeth, V3_SWAP_EXACT_IN_COMMAND, V3SwapExactIn, V3SwapExactInV2_1, V4_SWAP_COMMAND, WRAP_ETH_COMMAND,
        },
        contracts::{
            IUniversalRouter,
            v4::{IPoolManager, PoolKey},
        },
        deployment::{UniversalRouterAbi, get_provider_by_chain_contract, v3, v4},
        path::decode_path,
    },
};
use primitives::{AssetId, Chain, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata, decode_hex};

use super::{EVENT_WORD_SIZE, ParseContext, ParseContextExt, TransactionParser, ethereum_value_from_log_data};

const WITHDRAWAL_TOPIC: &str = "0x7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65";

pub struct UniversalRouterParser;

#[derive(Clone, Copy)]
struct RouterAbi {
    router: Option<Address>,
    v3: UniversalRouterAbi,
    v4: Option<UniversalRouterAbi>,
}

impl RouterAbi {
    fn from_chain_contract(chain: &Chain, contract: &str) -> Self {
        let v3_abi = v3::get_universal_router_abi_by_chain_contract(chain, contract);
        let v4_abi = v4::get_universal_router_abi_by_chain_contract(chain, contract);

        Self {
            router: Address::from_str(contract).ok(),
            v3: v3_abi.or(v4_abi).unwrap_or(UniversalRouterAbi::V2),
            v4: v4_abi,
        }
    }
}

impl TransactionParser<ParseContext<'_>, PrimitivesTransaction> for UniversalRouterParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context
            .transaction
            .to
            .as_ref()
            .is_some_and(|to| get_provider_by_chain_contract(context.metadata.chain, to).is_some())
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let to = context.transaction.to.as_ref()?;
        let provider = get_provider_by_chain_contract(context.metadata.chain, to)?;
        let input_bytes = decode_hex(&context.transaction.input).ok()?;
        let execute_call = IUniversalRouter::executeCall::abi_decode(&input_bytes).ok()?;
        let router_abi = RouterAbi::from_chain_contract(context.metadata.chain, to);
        let metadata = decode_execute_swap_call(
            context.metadata.chain,
            router_abi,
            &provider,
            &context.transaction.from,
            &execute_call,
            context.metadata.receipt,
        )?;

        context.make_swap_transaction(&context.transaction.from, &context.transaction.from, &metadata)
    }
}

pub(crate) fn decode_execute_swap(
    chain: &Chain,
    universal_router_abi: UniversalRouterAbi,
    provider: &str,
    from: &str,
    input_bytes: &[u8],
    receipt: &TransactionReceipt,
) -> Option<TransactionSwapMetadata> {
    let execute_call = IUniversalRouter::executeCall::abi_decode(input_bytes).ok()?;
    decode_execute_swap_call(
        chain,
        RouterAbi {
            router: None,
            v3: universal_router_abi,
            v4: None,
        },
        provider,
        from,
        &execute_call,
        receipt,
    )
}

fn decode_execute_swap_call(
    chain: &Chain,
    router_abi: RouterAbi,
    provider: &str,
    from: &str,
    execute_call: &IUniversalRouter::executeCall,
    receipt: &TransactionReceipt,
) -> Option<TransactionSwapMetadata> {
    let commands = &execute_call.commands;
    let inputs = &execute_call.inputs;
    let mut swap_input: Option<(AssetId, U256)> = None;
    let mut swap_output = None;
    let mut swap_provider = provider;

    let has_wrap = commands.contains(&WRAP_ETH_COMMAND);
    let mut unwrap_minimum = None;
    let mut sweep_minimum = None;

    for (command, input) in commands.iter().zip(inputs.iter()) {
        if command == &UNWRAP_WETH_COMMAND {
            let unwrap_weth = UnwrapWeth::abi_decode(input).ok()?;
            unwrap_minimum = Some(unwrap_weth.amount_min);
        } else if command == &SWEEP_COMMAND {
            let sweep = Sweep::abi_decode(input).ok()?;
            sweep_minimum = Some(sweep.amount_min);
        }
    }

    for (command, input) in commands.iter().zip(inputs.iter()) {
        if command == &V3_SWAP_EXACT_IN_COMMAND {
            let (amount_in, amount_out_min, path) = match router_abi.v3 {
                UniversalRouterAbi::V2 => {
                    let swap_exact_in = V3SwapExactIn::abi_decode(input).ok()?;
                    (swap_exact_in.amount_in, swap_exact_in.amount_out_min, swap_exact_in.path)
                }
                UniversalRouterAbi::V2_1 => {
                    let swap_exact_in = V3SwapExactInV2_1::abi_decode(input).ok()?;
                    (swap_exact_in.amount_in, swap_exact_in.amount_out_min, swap_exact_in.path)
                }
            };
            let token_pair = decode_path(&path)?;
            let from_token = token_pair.token_in.to_checksum(None);
            let to_token = token_pair.token_out.to_checksum(None);

            let leg_from_asset = AssetId::from(*chain, (!has_wrap).then_some(from_token));
            let (leg_to_asset, leg_to_value) = if let Some(unwrap_minimum) = &unwrap_minimum {
                (
                    AssetId::from_chain(*chain),
                    withdraw_value_from_receipt(&to_token, receipt).unwrap_or_else(|| unwrap_minimum.to_string()),
                )
            } else {
                let to_value = if let Some(sweep_minimum) = &sweep_minimum {
                    transfer_value_from_receipt(from, &to_token, receipt).unwrap_or_else(|| sweep_minimum.to_string())
                } else {
                    transfer_value_from_receipt(from, &to_token, receipt).unwrap_or_else(|| amount_out_min.to_string())
                };
                (AssetId::from(*chain, Some(to_token)), to_value)
            };
            match &mut swap_input {
                Some((from_asset, from_value)) if from_asset == &leg_from_asset => *from_value = from_value.checked_add(amount_in)?,
                None => swap_input = Some((leg_from_asset, amount_in)),
                Some(_) => {}
            }
            swap_output = Some((leg_to_asset, leg_to_value));
        }
        if command == &V4_SWAP_COMMAND
            && let Some(universal_router_abi) = router_abi.v4
            && let Ok(actions) = decode_action_data(input, universal_router_abi)
        {
            let native_output = if sweep_minimum.is_none() && commands.as_ref() == [V4_SWAP_COMMAND] {
                router_abi.router.and_then(|router| native_v4_value_from_receipt(chain, router, from, &actions, receipt))
            } else {
                None
            };
            for action in actions {
                let (from_token, to_token, amount_in) = match action {
                    V4Action::SWAP_EXACT_IN(params) => (params.currencyIn, params.path.last().map(|path_key| path_key.intermediateCurrency), params.amountIn),
                    V4Action::SWAP_EXACT_IN_V2_1(params) => (params.currencyIn, params.path.last().map(|path_key| path_key.intermediateCurrency), params.amountIn),
                    _ => continue,
                };
                let Some(to_token) = to_token else {
                    continue;
                };
                let leg_from_asset = AssetId::from(*chain, (!from_token.is_zero()).then(|| from_token.to_checksum(None)));
                let to_token_id = (!to_token.is_zero()).then(|| to_token.to_checksum(None));
                let leg_to_value = if let Some(to_token_id) = &to_token_id {
                    transfer_value_from_receipt(from, to_token_id, receipt)?
                } else {
                    native_output.as_ref().or(sweep_minimum.as_ref())?.to_string()
                };
                let leg_to_asset = AssetId::from(*chain, to_token_id);
                let leg_from_value = U256::from(amount_in);
                match &mut swap_input {
                    Some((from_asset, from_value)) if from_asset == &leg_from_asset => *from_value = from_value.checked_add(leg_from_value)?,
                    None => swap_input = Some((leg_from_asset, leg_from_value)),
                    Some(_) => {}
                }
                swap_output = Some((leg_to_asset, leg_to_value));
                swap_provider = SwapProvider::UniswapV4.id();
            }
        }
    }

    let (from_asset, from_value) = swap_input?;
    let (to_asset, to_value) = swap_output?;
    Some(TransactionSwapMetadata {
        from_asset,
        to_asset,
        from_value: u256_to_biguint(&from_value),
        to_value: u256_to_biguint(&U256::from_str(&to_value).ok()?),
        provider: Some(swap_provider.to_string()),
    })
}

fn native_v4_value_from_receipt(chain: &Chain, router: Address, from: &str, actions: &[V4Action], receipt: &TransactionReceipt) -> Option<U256> {
    let [swap, settlement, take] = actions else { return None };
    let (currency_in, path) = match swap {
        V4Action::SWAP_EXACT_IN(params) => (params.currencyIn, &params.path),
        V4Action::SWAP_EXACT_IN_V2_1(params) => (params.currencyIn, &params.path),
        _ => return None,
    };
    let (last, preceding) = path.split_last()?;
    if currency_in.is_zero()
        || !last.intermediateCurrency.is_zero()
        || path.iter().any(|hop| !hop.hooks.is_zero())
        || preceding.iter().any(|hop| hop.intermediateCurrency.is_zero())
    {
        return None;
    }
    match settlement {
        V4Action::SETTLE {
            currency, payer_is_user: true, ..
        }
        | V4Action::SETTLE_ALL { currency, .. }
            if *currency == currency_in => {}
        _ => return None,
    }
    match take {
        V4Action::TAKE { currency, recipient, amount }
            if currency.is_zero() && amount.is_zero() && (*recipient == Address::from_str(from).ok()? || *recipient == Address::from_str(MSG_SENDER).ok()?) => {}
        V4Action::TAKE_ALL { currency, .. } if currency.is_zero() => {}
        _ => return None,
    }
    let pool = PoolKey {
        currency0: Address::ZERO,
        currency1: preceding.last().map_or(currency_in, |hop| hop.intermediateCurrency),
        fee: last.fee,
        tickSpacing: last.tickSpacing,
        hooks: last.hooks,
    };
    let pool_id = keccak256(pool.abi_encode());
    let manager = v4::get_uniswap_deployment_by_chain(chain)?.pool_manager;
    let mut swaps = receipt.logs.iter().filter_map(|log| {
        if !log.address.eq_ignore_ascii_case(manager) {
            return None;
        }
        let topics = log.topics.iter().map(|topic| B256::from_str(topic)).collect::<Result<Vec<_>, _>>().ok()?;
        let event = IPoolManager::Swap::decode_raw_log_validate(topics, &decode_hex(&log.data).ok()?).ok()?;
        (event.id == pool_id && event.sender == router).then_some(event)
    });
    let swap = swaps.next()?;
    if swaps.next().is_some() || swap.amount0 <= 0 || swap.amount1 >= 0 {
        return None;
    }
    Some(U256::from(u128::try_from(swap.amount0).ok()?))
}

fn withdraw_value_from_receipt(token: &str, receipt: &TransactionReceipt) -> Option<String> {
    let token = ethereum_address_checksum(token).ok()?;

    receipt.logs.iter().find_map(|log| {
        (ethereum_address_checksum(&log.address).ok()? == token && log.topics.len() == 2 && log.topics.first().is_some_and(|topic| topic == WITHDRAWAL_TOPIC))
            .then(|| ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE))
            .flatten()
            .map(|value| value.to_string())
    })
}

fn transfer_value_from_receipt(to: &str, token: &str, receipt: &TransactionReceipt) -> Option<String> {
    let to = ethereum_address_checksum(to).ok()?;
    let token = ethereum_address_checksum(token).ok()?;

    receipt
        .logs
        .iter()
        .filter_map(|log| {
            (ethereum_address_checksum(&log.address).ok()? == token
                && log.topics.len() == 3
                && log.topics.first().is_some_and(|topic| topic == TRANSFER_TOPIC)
                && ethereum_address_from_topic(log.topics.get(2)?)? == to)
                .then(|| ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE))
                .flatten()
        })
        .reduce(|total, value| total + value)
        .map(|value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TOKEN_USDC_ADDRESS;
    use crate::rpc::model::{Transaction, TransactionReceipt};
    use crate::rpc::parsers::ProtocolParsers;
    use crate::uniswap::{actions::V4Action, contracts::v4::IV4Router, deployment::UniversalRouterAbi};
    use alloy_primitives::Address;
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain, TransactionSwapMetadata, TransactionType,
        asset_constants::{ETHEREUM_USDT_ASSET_ID, POLYGON_USDT_TOKEN_ID, UNICHAIN_DAI_TOKEN_ID, UNICHAIN_USDC_TOKEN_ID},
        contract_constants::{ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT, UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT},
        testkit::json_rpc::load_json_rpc_result,
    };

    fn map_swap(chain: &Chain, transaction: &Transaction, receipt: &TransactionReceipt) -> primitives::Transaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, DateTime::default()).unwrap()
    }

    #[test]
    fn test_map_v4_swap_cake_bnb() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_cake_bnb_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v4_cake_bnb_transaction_receipt.json"));
        let swap_transaction = map_swap(&Chain::SmartChain, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_transaction.metadata.unwrap()).unwrap();
        assert_eq!(swap_transaction.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.from_value, BigUint::from(1_000_000_000_000_000_000u64));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(metadata.to_value, BigUint::from(2_893_729_657_423_135u64));
        assert_eq!(metadata.provider.as_deref(), Some("uniswap_v4"));
    }

    #[test]
    fn test_map_v4_native_swap_rejects_unverified_events() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_cake_bnb_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v4_cake_bnb_transaction_receipt.json"));
        let original = receipt.logs[0].clone();
        let mut wrong_manager = original.clone();
        wrong_manager.address = transaction.from.clone();
        let mut wrong_pool = original.clone();
        wrong_pool.topics[1] = B256::ZERO.to_string();
        let mut wrong_router = original.clone();
        wrong_router.topics[2] = B256::ZERO.to_string();
        let mut wrong_signature = original.clone();
        wrong_signature.topics[0] = B256::ZERO.to_string();
        let mut malformed = original.clone();
        malformed.data = "0x01".to_string();
        let mut negative_output = original.clone();
        negative_output.data.replace_range(2..66, &"f".repeat(64));
        for invalid in [wrong_manager, wrong_pool, wrong_router, wrong_signature, malformed, negative_output] {
            let mut receipt = receipt.clone();
            receipt.logs[0] = invalid;
            assert_eq!(ProtocolParsers::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default()), None);
        }
        let mut duplicate = receipt.clone();
        duplicate.logs.push(original);
        assert_eq!(ProtocolParsers::map_transaction(&Chain::SmartChain, &transaction, &duplicate, DateTime::default()), None);
    }

    #[test]
    fn test_v4_native_payout_validation() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_cake_bnb_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v4_cake_bnb_transaction_receipt.json"));
        let execute = IUniversalRouter::executeCall::abi_decode(&decode_hex(&transaction.input).unwrap()).unwrap();
        let router = Address::from_str(transaction.to.as_ref().unwrap()).unwrap();
        let mut actions = decode_action_data(&execute.inputs[0], UniversalRouterAbi::V2_1).unwrap();
        for recipient in [Address::from_str(&transaction.from).unwrap(), Address::from_str(MSG_SENDER).unwrap()] {
            actions[2] = V4Action::TAKE {
                currency: Address::ZERO,
                recipient,
                amount: U256::ZERO,
            };
            assert_eq!(
                native_v4_value_from_receipt(&Chain::SmartChain, router, &transaction.from, &actions, &receipt),
                Some(U256::from(2_893_729_657_423_135u64))
            );
        }
        actions[2] = V4Action::TAKE_ALL {
            currency: Address::ZERO,
            min_amount: U256::ZERO,
        };
        assert_eq!(
            native_v4_value_from_receipt(&Chain::SmartChain, router, &transaction.from, &actions, &receipt),
            Some(U256::from(2_893_729_657_423_135u64))
        );
        for take in [
            V4Action::TAKE {
                currency: Address::ZERO,
                recipient: router,
                amount: U256::ZERO,
            },
            V4Action::TAKE {
                currency: Address::ZERO,
                recipient: Address::from_str(&transaction.from).unwrap(),
                amount: U256::from(1),
            },
            V4Action::TAKE_PORTION {
                currency: Address::ZERO,
                recipient: router,
                bips: U256::from(50),
            },
        ] {
            actions[2] = take;
            assert_eq!(native_v4_value_from_receipt(&Chain::SmartChain, router, &transaction.from, &actions, &receipt), None);
        }
        actions[2] = V4Action::TAKE_ALL {
            currency: Address::ZERO,
            min_amount: U256::ZERO,
        };
        if let V4Action::SWAP_EXACT_IN_V2_1(params) = &mut actions[0] {
            params.path[0].hooks = router;
        }
        assert_eq!(native_v4_value_from_receipt(&Chain::SmartChain, router, &transaction.from, &actions, &receipt), None);
    }

    #[test]
    fn test_map_v4_swap_eth_dai() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_eth_dai_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v4_eth_dai_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Unichain, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, BigUint::from(1000000000000000u64));

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None
            }
        );
        assert_eq!(metadata.from_value, BigUint::from(995000000000000u64));
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some(UNICHAIN_DAI_TOKEN_ID.to_string())
            }
        );
        assert_eq!(metadata.to_value, BigUint::parse_bytes(b"2696771430516915192", 10).unwrap());
    }

    #[test]
    fn test_map_v4_swap_usdc_eth() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v4_usdc_eth_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v4_usdc_eth_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Unichain, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.to, "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(swap_tx.contract.unwrap(), UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Unichain));
        assert_eq!(swap_tx.value, BigUint::from(0u64));

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: Some(UNICHAIN_USDC_TOKEN_ID.to_string())
            }
        );
        assert_eq!(metadata.from_value, BigUint::from(2132953u64));
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Unichain,
                token_id: None
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(1155057703771482u64));
    }

    #[test]
    fn test_map_split_v3_v4_swap_eth_usdt() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_v4_eth_usdt_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v3_v4_eth_usdt_transaction_receipt.json"));

        let swap_transaction = map_swap(&Chain::Ethereum, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_transaction.metadata.unwrap()).unwrap();

        assert_eq!(swap_transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_transaction.value, BigUint::from(10000000000000u64));
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(metadata.from_value, BigUint::from(10000000000000u64));
        assert_eq!(metadata.to_asset, ETHEREUM_USDT_ASSET_ID.clone());
        assert_eq!(metadata.to_value, BigUint::from(19304u64));
    }

    #[test]
    fn test_map_v3_swap_eth_token() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_eth_token_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v3_eth_token_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Ethereum, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.to, "0x10E11c7368552D5Ab9ef5eED496f614fBAAe9F0D");
        assert_eq!(swap_tx.contract.unwrap(), ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, BigUint::from(18000000000000000u64));

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: None
            }
        );
        assert_eq!(metadata.from_value, BigUint::from(17910000000000000u64));
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0xcf0C122c6b73ff809C693DB761e7BaeBe62b6a2E".to_string())
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(512854887193301u64));
    }

    #[test]
    fn test_map_v3_swap_token_eth() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_token_eth_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v3_token_eth_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Base, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x985Cf24b63a98510298997Af83a31D8625C09bA5");
        assert_eq!(swap_tx.to, "0x985Cf24b63a98510298997Af83a31D8625C09bA5");
        assert_eq!(swap_tx.contract.unwrap(), "0xFE6508f0015C778Bdcc1fB5465bA5ebE224C9912");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Base));
        assert_eq!(swap_tx.value, BigUint::from(0u64));

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x532f27101965dd16442E59d40670FaF5eBB142E4".to_string())
            }
        );
        assert_eq!(metadata.from_value, BigUint::parse_bytes(b"1352497738700000000000", 10).unwrap());
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: None
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(29020434785385862u64));
    }

    #[test]
    fn test_map_v3_swap_pol_usdt() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_pol_usdt_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v3_pol_usdt_tx_receipt.json"));

        let swap_tx = map_swap(&Chain::Polygon, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0x8f4b6cbF3373e065aEb3FEc6027Ff8Ca9a665DE2");
        assert_eq!(swap_tx.to, "0x8f4b6cbF3373e065aEb3FEc6027Ff8Ca9a665DE2");
        assert_eq!(swap_tx.contract.unwrap(), "0xec7BE89e9d109e7e3Fec59c222CF297125FEFda2");
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Polygon));
        assert_eq!(swap_tx.value, BigUint::parse_bytes(b"372000000000000000000", 10).unwrap());

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Polygon,
                token_id: None
            }
        );
        assert_eq!(metadata.from_value, BigUint::parse_bytes(b"372000000000000000000", 10).unwrap());
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Polygon,
                token_id: Some(POLYGON_USDT_TOKEN_ID.to_string())
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(78290151u64));
    }

    #[test]
    fn test_map_v3_swap_usdc_paxg() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/v3_usdc_paxg_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../../testdata/v3_usdc_paxg_receipt.json"));

        let swap_tx = map_swap(&Chain::Ethereum, &transaction, &receipt);
        let metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.unwrap()).unwrap();

        assert_eq!(swap_tx.from, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.to, "0xBa38FE5b73eA5b93d0733CF9eb10aDea6E1E3a2a");
        assert_eq!(swap_tx.contract.unwrap(), ETHEREUM_UNISWAP_V3_UNIVERSAL_ROUTER_CONTRACT);
        assert_eq!(swap_tx.transaction_type, TransactionType::Swap);
        assert_eq!(swap_tx.fee_asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(swap_tx.value, BigUint::from(0u64));

        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some(TOKEN_USDC_ADDRESS.to_string())
            }
        );
        assert_eq!(metadata.from_value, BigUint::from(29850000u64));
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Ethereum,
                token_id: Some("0x45804880De22913dAFE09f4980848ECE6EcbAf78".to_string())
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(9017156750431593u64));
    }

    #[test]
    fn test_v4_swap_empty_path_no_panic() {
        let action = V4Action::SWAP_EXACT_IN(IV4Router::ExactInputParams {
            currencyIn: Address::ZERO,
            path: vec![],
            amountIn: 1000000000000000000_u128,
            amountOutMinimum: 0,
        });

        let encoded_actions = crate::uniswap::actions::encode_actions(&[action]);
        let decoded_actions = crate::uniswap::actions::decode_action_data(&encoded_actions, UniversalRouterAbi::V2);
        assert!(decoded_actions.is_ok());

        let actions = decoded_actions.unwrap();
        assert_eq!(actions.len(), 1);

        if let V4Action::SWAP_EXACT_IN(params) = &actions[0] {
            assert!(params.path.is_empty());
        }
    }
}
