use num_bigint::BigUint;
use primitives::{AssetAddress, AssetBalance, ChainAddress};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub struct AssetAddressChanges {
    pub addresses_to_add: Vec<AssetAddress>,
    pub addresses_to_delete: Vec<AssetAddress>,
}

impl AssetAddressChanges {
    pub fn from_coin_balance(chain_address: &ChainAddress, balance: AssetBalance) -> Self {
        let asset_address = AssetAddress::new(balance.asset_id, chain_address.address.clone(), Some(balance.balance.available.to_string()));

        if balance.balance.available == BigUint::ZERO {
            Self {
                addresses_to_add: vec![],
                addresses_to_delete: vec![asset_address],
            }
        } else {
            Self {
                addresses_to_add: vec![asset_address],
                addresses_to_delete: vec![],
            }
        }
    }

    pub fn from_token_balances(chain_address: &ChainAddress, existing_addresses: Vec<AssetAddress>, latest_balances: Vec<AssetBalance>) -> Self {
        let mut seen = HashSet::new();
        let addresses_to_add: Vec<_> = latest_balances
            .into_iter()
            .filter(|asset| asset.asset_id.token_id.is_some())
            .filter(|asset| seen.insert(asset.asset_id.clone()))
            .filter(|asset| asset.balance.available > BigUint::ZERO)
            .map(|asset| AssetAddress::new(asset.asset_id, chain_address.address.clone(), Some(asset.balance.available.to_string())))
            .collect();

        let added_ids: HashSet<_> = addresses_to_add.iter().map(|address| address.asset_id.clone()).collect();
        let addresses_to_delete = existing_addresses
            .into_iter()
            .filter(|address| address.asset_id.token_id.is_some())
            .filter(|address| !added_ids.contains(&address.asset_id))
            .collect();

        Self {
            addresses_to_add,
            addresses_to_delete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain};

    #[test]
    fn test_asset_address_changes_from_coin_balance() {
        let chain_address = ChainAddress::new(Chain::Ethereum, "0xwallet".to_string());
        let positive_balance = AssetBalance::new(Asset::mock_eth().id.clone(), BigUint::from(10u32));
        let positive_changes = AssetAddressChanges::from_coin_balance(&chain_address, positive_balance);

        assert_eq!(
            positive_changes.addresses_to_add,
            vec![AssetAddress::new(Asset::mock_eth().id.clone(), chain_address.address.clone(), Some("10".to_string()))]
        );
        assert_eq!(positive_changes.addresses_to_delete, vec![]);

        let zero_balance = AssetBalance::new(Asset::mock_eth().id.clone(), BigUint::ZERO);
        let zero_changes = AssetAddressChanges::from_coin_balance(&chain_address, zero_balance);

        assert_eq!(zero_changes.addresses_to_add, vec![]);
        assert_eq!(
            zero_changes.addresses_to_delete,
            vec![AssetAddress::new(Asset::mock_eth().id, chain_address.address, Some("0".to_string()))]
        );
    }

    #[test]
    fn test_asset_address_changes_from_token_balances() {
        let chain_address = ChainAddress::new(Chain::Ethereum, "0xwallet".to_string());
        let existing_addresses = vec![
            AssetAddress::new(Asset::mock_eth().id, chain_address.address.clone(), Some("10".to_string())),
            AssetAddress::new(Asset::mock_ethereum_usdc().id.clone(), chain_address.address.clone(), Some("5".to_string())),
            AssetAddress::new(Asset::mock_erc20().id.clone(), chain_address.address.clone(), Some("7".to_string())),
        ];

        let omitted_zero_changes = AssetAddressChanges::from_token_balances(
            &chain_address,
            existing_addresses.clone(),
            vec![AssetBalance::new(Asset::mock_erc20().id.clone(), BigUint::from(9u32))],
        );
        assert_eq!(
            omitted_zero_changes.addresses_to_delete,
            vec![AssetAddress::new(Asset::mock_ethereum_usdc().id, chain_address.address.clone(), Some("5".to_string()))]
        );
        assert_eq!(
            omitted_zero_changes.addresses_to_add,
            vec![AssetAddress::new(Asset::mock_erc20().id, chain_address.address.clone(), Some("9".to_string()))]
        );

        let explicit_zero_changes = AssetAddressChanges::from_token_balances(
            &chain_address,
            existing_addresses,
            vec![
                AssetBalance::new(Asset::mock_ethereum_usdc().id.clone(), BigUint::ZERO),
                AssetBalance::new(Asset::mock_erc20().id.clone(), BigUint::from(9u32)),
            ],
        );
        assert_eq!(
            explicit_zero_changes.addresses_to_delete,
            vec![AssetAddress::new(Asset::mock_ethereum_usdc().id, chain_address.address.clone(), Some("5".to_string()))]
        );
        assert_eq!(
            explicit_zero_changes.addresses_to_add,
            vec![AssetAddress::new(Asset::mock_erc20().id, chain_address.address.clone(), Some("9".to_string()))]
        );

        let new_zero_changes = AssetAddressChanges::from_token_balances(&chain_address, vec![], vec![AssetBalance::new(Asset::mock_ethereum_usdc().id.clone(), BigUint::ZERO)]);
        assert_eq!(new_zero_changes.addresses_to_delete, vec![]);
        assert_eq!(new_zero_changes.addresses_to_add, vec![]);
    }
}
