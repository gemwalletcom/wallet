use std::collections::HashMap;

use alloy_sol_types::SolCall;
use gem_evm::across::{contracts::V3SpokePoolInterface, deployment::AcrossDeployment};
use num_bigint::BigUint;
use primitives::{Chain, ChainType, Transaction, decode_hex};

use crate::SwapperProvider;
use crate::mayan::MAYAN_FORWARDER_SELECTORS;
use crate::thorchain::{contracts::RouterInterface, memo::ThorchainMemo};

#[derive(Debug, serde::Serialize)]
pub struct VaultAddresses {
    pub deposit: Vec<String>,
    pub send: Vec<String>,
}

pub type DepositAddressMap = HashMap<String, SwapperProvider>;
pub type SendAddressMap = HashMap<String, SwapperProvider>;

const CHAINFLIP_SWAP_SELECTORS: [&str; 2] = ["0xdd687345", "0x04fc7da0"];

pub(crate) fn rescale_base_units(amount: BigUint, from_decimals: u32, to_decimals: u32) -> BigUint {
    match to_decimals.cmp(&from_decimals) {
        std::cmp::Ordering::Equal => amount,
        std::cmp::Ordering::Greater => amount * BigUint::from(10_u32).pow(to_decimals - from_decimals),
        std::cmp::Ordering::Less => amount / BigUint::from(10_u32).pow(from_decimals - to_decimals),
    }
}

pub fn swap_provider_with_vault_addresses(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> Option<SwapperProvider> {
    deposit_address_provider(transaction, deposit_addresses).filter(|provider| is_valid_swap_transaction(provider, transaction))
}

pub fn in_transit_swap_provider(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> Option<SwapperProvider> {
    transaction
        .swap_metadata()
        .and_then(|metadata| metadata.provider?.parse::<SwapperProvider>().ok())
        .or_else(|| deposit_addresses.get(&transaction.id.to_string()).copied())
        .or_else(|| deposit_address_provider(transaction, deposit_addresses))
}

fn deposit_address_provider(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> Option<SwapperProvider> {
    deposit_addresses
        .get(&transaction.to)
        .copied()
        .or_else(|| transaction.output_addresses().into_iter().find_map(|address| deposit_addresses.get(&address).copied()))
        .or_else(|| deposit_addresses.get(&deposit_order_id(transaction)?).copied())
        .or_else(|| deposit_addresses.get(transaction.memo.as_deref()?).copied())
        .or_else(|| deposit_addresses.get(transaction.data.as_deref()?).copied())
        .or_else(|| deposit_addresses.get(transaction.contract.as_deref()?).copied().filter(|provider| *provider == SwapperProvider::Chainflip))
}

fn deposit_order_id(transaction: &Transaction) -> Option<String> {
    let arguments = transaction.data.as_deref()?.strip_prefix("0x")?.get(8..)?;
    let order_id = arguments.get(arguments.len().checked_sub(64)?..)?;
    Some(format!("0x{order_id}"))
}

fn is_valid_swap_transaction(provider: &SwapperProvider, transaction: &Transaction) -> bool {
    match provider {
        SwapperProvider::Thorchain | SwapperProvider::Mayachain => thorchain_memo(transaction).is_some_and(|memo| ThorchainMemo::is_swap(&memo)),
        SwapperProvider::Chainflip => is_valid_chainflip_swap(transaction),
        SwapperProvider::Across => is_across_deposit(transaction),
        SwapperProvider::Mayan => is_valid_mayan_swap(transaction),
        _ => true,
    }
}

fn thorchain_memo(transaction: &Transaction) -> Option<String> {
    transaction.memo.clone().or_else(|| {
        let data = decode_hex(transaction.data.as_deref()?).ok()?;
        Some(RouterInterface::depositWithExpiryCall::abi_decode(&data).ok()?.memo)
    })
}

fn is_across_deposit(transaction: &Transaction) -> bool {
    transaction
        .data
        .as_deref()
        .and_then(|data| V3SpokePoolInterface::depositV3Call::abi_decode(&decode_hex(data).ok()?).ok())
        .and_then(|deposit| Chain::from_chain_id(u64::try_from(deposit.destinationChainId).ok()?))
        .is_some_and(|destination| AcrossDeployment::deployment_by_chain(&destination).is_some())
}

fn is_valid_mayan_swap(transaction: &Transaction) -> bool {
    if transaction.asset_id.chain.chain_type() != ChainType::Ethereum {
        return true;
    }

    transaction
        .data
        .as_deref()
        .and_then(|data| decode_hex(data.get(..10)?).ok())
        .is_some_and(|selector| MAYAN_FORWARDER_SELECTORS.iter().any(|forwarder| selector == forwarder))
}

fn is_valid_chainflip_swap(transaction: &Transaction) -> bool {
    match transaction.asset_id.chain.chain_type() {
        ChainType::Ethereum => transaction
            .data
            .as_deref()
            .and_then(|data| data.get(..10))
            .is_some_and(|selector| CHAINFLIP_SWAP_SELECTORS.iter().any(|candidate| selector.eq_ignore_ascii_case(candidate))),
        ChainType::Tron => transaction.memo.as_deref().is_some_and(|note| !note.is_empty()),
        _ => true,
    }
}

pub fn is_cross_chain_swap(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> bool {
    swap_provider_with_vault_addresses(transaction, deposit_addresses).is_some()
}

pub fn is_from_vault_address(transaction: &Transaction, send_addresses: &SendAddressMap) -> bool {
    send_addresses.contains_key(&transaction.from) || transaction.input_addresses().iter().any(|address| send_addresses.contains_key(address))
}

pub fn is_received_from_vault(transaction: &Transaction, address: &str, send_addresses: &SendAddressMap) -> bool {
    !transaction.is_sent(address.to_string()) && is_from_vault_address(&transaction.finalize(vec![address.to_string()]), send_addresses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, Bytes, U256, hex};
    use gem_evm::rpc::{
        mapper::EthereumMapper,
        model::{Transaction as EvmTransaction, TransactionReceipt},
    };
    use primitives::{
        AssetId, TransactionSwapMetadata, TransactionType, TransactionUtxoInput, contract_constants::SOLANA_MAYAN_CPI_PROXY_PROGRAM_ID, testkit::json_rpc::load_json_rpc_result, transaction_metadata_types::TransactionAssetTransfer,
    };

    #[test]
    fn test_vault_address_detected() {
        let vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_deposit_order_id_detected() {
        let order_id = "0x5018b8125a7aec15f5a9e8bf07786e79cc1b52b6d97fb1a92930fc5aeea6ddf0".to_string();
        let deposit_addresses = DepositAddressMap::from([(order_id, SwapperProvider::Relay)]);
        let deposit = Transaction {
            to: "0x4cD00E387622C35bDDB9b4c962C136462338BC31".to_string(),
            data: Some("0x49290c1c0000000000000000000000000d9dab1a248f63b0a48965ba8435e4de7497a3dc5018b8125a7aec15f5a9e8bf07786e79cc1b52b6d97fb1a92930fc5aeea6ddf0".to_string()),
            ..Transaction::mock()
        };
        let other_order = Transaction {
            data: Some("0x49290c1c0000000000000000000000000d9dab1a248f63b0a48965ba8435e4de7497a3dcf03274723d3ac56f800095018a0fb48d357d0af90bead4142cb134e06099bb13".to_string()),
            ..deposit.clone()
        };
        let selector_only = Transaction {
            data: Some("0x49290c1c".to_string()),
            ..deposit.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&deposit, &deposit_addresses), Some(SwapperProvider::Relay));
        assert_eq!(swap_provider_with_vault_addresses(&other_order, &deposit_addresses), None);
        assert_eq!(swap_provider_with_vault_addresses(&selector_only, &deposit_addresses), None);
    }

    #[test]
    fn test_memo_order_id_detected() {
        let order_id = "0xef469816498676be697de680c9ed346fbb40d4108b45851c18581170a5f6e5d9".to_string();
        let deposit_addresses = DepositAddressMap::from([(order_id.clone(), SwapperProvider::Relay)]);
        let deposit = Transaction {
            asset_id: AssetId::from_chain(Chain::Ton),
            to: "UQCrdGsDTqA2t6xRR4N6V4J705F7w_VQbUdHnofsh-8lVN4p".to_string(),
            memo: Some(order_id),
            ..Transaction::mock()
        };
        let other_memo = Transaction {
            memo: Some("gift".to_string()),
            ..deposit.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&deposit, &deposit_addresses), Some(SwapperProvider::Relay));
        assert_eq!(swap_provider_with_vault_addresses(&other_memo, &deposit_addresses), None);
    }

    #[test]
    fn test_calldata_deposit_detected() {
        let calldata = include_str!("bridgers/testdata/swap_base_usdc_bsc_usdt.json");
        let calldata = serde_json::from_str::<serde_json::Value>(calldata).unwrap()["data"]["txData"]["data"].as_str().unwrap().to_string();
        let deposit_addresses = DepositAddressMap::from([(calldata.clone(), SwapperProvider::Bridgers)]);
        let deposit = Transaction {
            to: "0xa18968cc31232724f1dbd0d1e8d0b323d89f3501".to_string(),
            data: Some(calldata),
            ..Transaction::mock()
        };
        let other_swap = Transaction {
            data: Some("0x9ddf93bb".to_string()),
            ..deposit.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&deposit, &deposit_addresses), Some(SwapperProvider::Bridgers));
        assert_eq!(swap_provider_with_vault_addresses(&other_swap, &deposit_addresses), None);
    }

    #[test]
    fn test_no_vault_address() {
        let empty = DepositAddressMap::new();
        assert!(!is_cross_chain_swap(&Transaction::mock(), &empty));
    }

    #[test]
    fn test_chainflip_evm_swap_methods() {
        let vault = "0xF5e10380213880111522dd0efD3dbb45b9f62Bcc".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Chainflip)]);

        let native_swap = Transaction {
            to: vault.clone(),
            data: Some("0xdd68734500000000".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider_with_vault_addresses(&native_swap, &deposit_addresses), Some(SwapperProvider::Chainflip));

        let token_swap = Transaction {
            to: vault.clone(),
            data: Some("0x04fc7da000000000".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider_with_vault_addresses(&token_swap, &deposit_addresses), Some(SwapperProvider::Chainflip));

        let all_batch = Transaction {
            to: vault.clone(),
            data: Some("0x5f8c0f9a00000000".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider_with_vault_addresses(&all_batch, &deposit_addresses), None);

        let unknown_method = Transaction {
            to: vault.clone(),
            data: Some("0xdeadbeef00000000".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(swap_provider_with_vault_addresses(&unknown_method, &deposit_addresses), None);

        let missing_data = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(swap_provider_with_vault_addresses(&missing_data, &deposit_addresses), None);
    }

    #[test]
    fn test_mayan_evm_forwarder_deposits() {
        let forwarder = "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2".to_string();
        let deposit_addresses = DepositAddressMap::from([(forwarder.clone(), SwapperProvider::Mayan)]);
        let forward_erc20 = Transaction {
            to: forwarder.clone(),
            data: Some(format!("0x{}00000000", hex::encode(MAYAN_FORWARDER_SELECTORS[0]))),
            ..Transaction::mock()
        };
        let mctp_fill = Transaction {
            data: Some("0xa088352600000000".to_string()),
            ..forward_erc20.clone()
        };
        let solana_deposit = Transaction {
            asset_id: AssetId::from_chain(Chain::Solana),
            data: None,
            ..forward_erc20.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&forward_erc20, &deposit_addresses), Some(SwapperProvider::Mayan));
        assert_eq!(swap_provider_with_vault_addresses(&mctp_fill, &deposit_addresses), None);
        assert_eq!(swap_provider_with_vault_addresses(&solana_deposit, &deposit_addresses), Some(SwapperProvider::Mayan));
    }

    #[test]
    fn test_across_deposits() {
        let spoke_pool = "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5".to_string();
        let deposit_addresses = DepositAddressMap::from([(spoke_pool.clone(), SwapperProvider::Across)]);
        let deposit_call = |destination_chain_id: u64| V3SpokePoolInterface::depositV3Call {
            depositor: Address::ZERO,
            recipient: Address::ZERO,
            inputToken: Address::ZERO,
            outputToken: Address::ZERO,
            inputAmount: U256::from(1),
            outputAmount: U256::from(1),
            destinationChainId: U256::from(destination_chain_id),
            exclusiveRelayer: Address::ZERO,
            quoteTimestamp: 0,
            fillDeadline: 0,
            exclusivityDeadline: 0,
            message: Bytes::new(),
        };
        let deposit = Transaction {
            to: spoke_pool.clone(),
            data: Some(hex::encode_prefixed(deposit_call(8453).abi_encode())),
            ..Transaction::mock()
        };
        let unsupported_destination = Transaction {
            data: Some(hex::encode_prefixed(deposit_call(4217).abi_encode())),
            ..deposit.clone()
        };
        let fill = Transaction {
            data: Some("0xdeff4b2400000000".to_string()),
            ..deposit.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&deposit, &deposit_addresses), Some(SwapperProvider::Across));
        assert_eq!(swap_provider_with_vault_addresses(&unsupported_destination, &deposit_addresses), None);
        assert_eq!(swap_provider_with_vault_addresses(&fill, &deposit_addresses), None);
    }

    #[test]
    fn test_in_transit_swap_provider() {
        let vault = "0xF5e10380213880111522dd0efD3dbb45b9f62Bcc".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Chainflip)]);
        let stored_chainflip_swap = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(in_transit_swap_provider(&stored_chainflip_swap, &deposit_addresses), Some(SwapperProvider::Chainflip));

        let metadata = TransactionSwapMetadata::new(AssetId::from_chain(Chain::Polygon), 1u32.into(), AssetId::from_chain(Chain::Base), 1u32.into(), SwapperProvider::Across);
        let across_deposit = Transaction {
            metadata: serde_json::to_value(metadata).ok(),
            ..Transaction::mock()
        };
        assert_eq!(in_transit_swap_provider(&across_deposit, &deposit_addresses), Some(SwapperProvider::Across));

        assert_eq!(in_transit_swap_provider(&Transaction::mock(), &deposit_addresses), None);

        let relay_deposit = Transaction::mock();
        let deposit_ids = DepositAddressMap::from([(relay_deposit.id.to_string(), SwapperProvider::Relay)]);
        assert_eq!(in_transit_swap_provider(&relay_deposit, &deposit_ids), Some(SwapperProvider::Relay));
    }

    #[test]
    fn test_chainflip_tron_vault_swap() {
        let vault = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Chainflip)]);
        let vault_swap = Transaction {
            asset_id: AssetId::from_chain(Chain::Tron),
            to: vault,
            memo: Some("0x0300".to_string()),
            ..Transaction::mock()
        };
        let dust = Transaction { memo: None, ..vault_swap.clone() };

        assert_eq!(swap_provider_with_vault_addresses(&vault_swap, &deposit_addresses), Some(SwapperProvider::Chainflip));
        assert_eq!(swap_provider_with_vault_addresses(&dust, &deposit_addresses), None);
    }

    #[test]
    fn test_chainflip_non_evm_vault_swap() {
        let vault = "J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Chainflip)]);
        let transaction = Transaction {
            asset_id: AssetId::from_chain(Chain::Solana),
            to: vault,
            ..Transaction::mock()
        };

        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::Chainflip));
    }

    #[test]
    fn test_chainflip_solana_vault_program_call() {
        let vault = "J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT".to_string();
        let sender = "CabroWmzUzcqqGvprUoC7RnJznuwX6qf5W1tSSaomri7".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Chainflip)]);
        let transaction = Transaction {
            asset_id: AssetId::from_chain(Chain::Solana),
            from: sender.clone(),
            to: sender,
            contract: Some(vault),
            transaction_type: TransactionType::SmartContractCall,
            ..Transaction::mock()
        };

        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::Chainflip));
        assert_eq!(in_transit_swap_provider(&transaction, &deposit_addresses), Some(SwapperProvider::Chainflip));
    }

    #[test]
    fn test_program_call_to_other_provider_deposit_address() {
        let sender = "BnPkG7QaDZQ4k4m2sPYb5mVEA2GgpphY9kgzWAHxjtvC".to_string();
        let deposit_addresses = DepositAddressMap::from([(SOLANA_MAYAN_CPI_PROXY_PROGRAM_ID.to_string(), SwapperProvider::Mayan)]);
        let transaction = Transaction {
            asset_id: AssetId::from_chain(Chain::Solana),
            from: sender.clone(),
            to: sender,
            contract: Some(SOLANA_MAYAN_CPI_PROXY_PROGRAM_ID.to_string()),
            transaction_type: TransactionType::SmartContractCall,
            ..Transaction::mock()
        };

        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), None);
    }

    #[test]
    fn test_thorchain_vault_with_swap_memo() {
        let vault = "bc1qvault".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction {
            to: vault,
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert!(is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_thorchain_vault_without_memo() {
        let vault = "bc1qvault".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert!(!is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_thorchain_vault_with_non_swap_memo() {
        let vault = "bc1qvault".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction {
            to: vault,
            memo: Some("ADD:ETH.ETH:0x123".to_string()),
            ..Transaction::mock()
        };
        assert!(!is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_thorchain_router_token_deposit() {
        let transaction = load_json_rpc_result::<EvmTransaction>(include_str!("thorchain/testdata/router_deposit_usdt_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("thorchain/testdata/router_deposit_usdt_tx_receipt.json"));
        let deposit = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, &BigUint::from(1792508567u64)).unwrap();
        let deposit_addresses = DepositAddressMap::from([("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string(), SwapperProvider::Thorchain)]);
        let other_call = Transaction {
            data: Some("0xdeadbeef00000000".to_string()),
            ..deposit.clone()
        };

        assert_eq!(swap_provider_with_vault_addresses(&deposit, &deposit_addresses), Some(SwapperProvider::Thorchain));
        assert_eq!(swap_provider_with_vault_addresses(&other_call, &deposit_addresses), None);
    }

    #[test]
    fn test_utxo_vault_address_in_outputs() {
        let vault = "vault_address".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new("sender".into(), 50000u32.into())],
            vec![TransactionUtxoInput::new(vault, 40000u32.into()), TransactionUtxoInput::new("change".into(), 9000u32.into())],
        );
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_utxo_no_vault_address_in_outputs() {
        let deposit_addresses = DepositAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(vec![TransactionUtxoInput::new("sender".into(), 50000u32.into())], vec![TransactionUtxoInput::new("recipient".into(), 40000u32.into())]);
        assert!(!is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_is_from_vault_address() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction { from: vault, ..Transaction::mock() };
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_from_vault_address_utxo() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(vec![TransactionUtxoInput::new(vault, 50000u32.into())], vec![TransactionUtxoInput::new("recipient".into(), 40000u32.into())]);
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_received_from_vault() {
        let send_addresses = SendAddressMap::from([("0xvault".to_string(), SwapperProvider::Mayan)]);
        let payout = Transaction::mock_with_asset_transfers(vec![TransactionAssetTransfer {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            from: "0xvault".to_string(),
            to: "0xuser".to_string(),
            value: 1u32.into(),
        }]);
        let own_swap = Transaction {
            from: "0xuser".to_string(),
            ..payout.clone()
        };
        let vault_transfer = Transaction {
            from: "0xvault".to_string(),
            ..Transaction::mock()
        };

        assert!(is_received_from_vault(&payout, "0xuser", &send_addresses));
        assert!(is_received_from_vault(&vault_transfer, "0xto", &send_addresses));
        assert!(!is_received_from_vault(&own_swap, "0xuser", &send_addresses));
        assert!(!is_received_from_vault(&payout, "0xother", &send_addresses));
    }

    #[test]
    fn test_is_not_from_vault_address() {
        let send_addresses = SendAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        assert!(!is_from_vault_address(&Transaction::mock(), &send_addresses));
    }
}
