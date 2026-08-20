use std::collections::HashMap;

use primitives::{ChainType, Transaction};

use crate::SwapperProvider;
use crate::thorchain::memo::ThorchainMemo;

#[derive(Debug, serde::Serialize)]
pub struct VaultAddresses {
    pub deposit: Vec<String>,
    pub send: Vec<String>,
}

pub type DepositAddressMap = HashMap<String, SwapperProvider>;
pub type SendAddressMap = HashMap<String, SwapperProvider>;

const CHAINFLIP_SWAP_SELECTORS: [&str; 2] = ["0xdd687345", "0x04fc7da0"];

pub fn swap_provider_with_vault_addresses(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> Option<SwapperProvider> {
    deposit_addresses
        .get(&transaction.to)
        .copied()
        .or_else(|| transaction.output_addresses().into_iter().find_map(|addr| deposit_addresses.get(&addr).copied()))
        .filter(|provider| is_valid_swap_transaction(provider, transaction))
}

fn is_valid_swap_transaction(provider: &SwapperProvider, transaction: &Transaction) -> bool {
    match provider {
        SwapperProvider::Thorchain | SwapperProvider::Mayachain => transaction.memo.as_deref().is_some_and(ThorchainMemo::is_swap),
        SwapperProvider::Chainflip => is_valid_chainflip_swap(transaction),
        _ => true,
    }
}

fn is_valid_chainflip_swap(transaction: &Transaction) -> bool {
    if transaction.asset_id.chain.chain_type() != ChainType::Ethereum {
        return true;
    }

    transaction
        .data
        .as_deref()
        .and_then(|data| data.get(..10))
        .is_some_and(|selector| CHAINFLIP_SWAP_SELECTORS.iter().any(|candidate| selector.eq_ignore_ascii_case(candidate)))
}

pub fn is_cross_chain_swap(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> bool {
    swap_provider_with_vault_addresses(transaction, deposit_addresses).is_some()
}

pub fn is_from_vault_address(transaction: &Transaction, send_addresses: &SendAddressMap) -> bool {
    send_addresses.contains_key(&transaction.from) || transaction.input_addresses().iter().any(|addr| send_addresses.contains_key(addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, TransactionUtxoInput};

    #[test]
    fn test_vault_address_detected() {
        let vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction { to: vault, ..Transaction::mock() };
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
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
    fn test_thorchain_router_with_swap_memo() {
        let vault = "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::Thorchain)]);
        let transaction = Transaction {
            to: vault,
            memo: Some("=:BTC:bc1qaddress:0/1/0:affiliate:150".to_string()),
            ..Transaction::mock()
        };
        assert!(is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_utxo_vault_address_in_outputs() {
        let vault = "vault_address".to_string();
        let deposit_addresses = DepositAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new("sender".into(), 50000u32.into())],
            vec![
                TransactionUtxoInput::new(vault, 40000u32.into()),
                TransactionUtxoInput::new("change".into(), 9000u32.into()),
            ],
        );
        assert_eq!(swap_provider_with_vault_addresses(&transaction, &deposit_addresses), Some(SwapperProvider::NearIntents));
    }

    #[test]
    fn test_utxo_no_vault_address_in_outputs() {
        let deposit_addresses = DepositAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new("sender".into(), 50000u32.into())],
            vec![TransactionUtxoInput::new("recipient".into(), 40000u32.into())],
        );
        assert!(!is_cross_chain_swap(&transaction, &deposit_addresses));
    }

    #[test]
    fn test_is_from_vault_address() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction {
            from: vault,
            ..Transaction::mock()
        };
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_from_vault_address_utxo() {
        let vault = "vault_address".to_string();
        let send_addresses = SendAddressMap::from([(vault.clone(), SwapperProvider::NearIntents)]);
        let transaction = Transaction::mock_utxo(
            vec![TransactionUtxoInput::new(vault, 50000u32.into())],
            vec![TransactionUtxoInput::new("recipient".into(), 40000u32.into())],
        );
        assert!(is_from_vault_address(&transaction, &send_addresses));
    }

    #[test]
    fn test_is_not_from_vault_address() {
        let send_addresses = SendAddressMap::from([("vault_address".to_string(), SwapperProvider::NearIntents)]);
        assert!(!is_from_vault_address(&Transaction::mock(), &send_addresses));
    }
}
