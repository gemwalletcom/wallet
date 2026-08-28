use std::collections::HashMap;

use crate::services::collections::{missing, unique};

use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Account, Asset, AssetBalance, AssetId, Chain};

use super::model::{GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};

#[derive(Debug, Clone, PartialEq)]
pub struct BalanceRequest {
    pub chain: Chain,
    pub address: String,
    pub coin: bool,
    pub token_ids: Vec<AssetId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BalanceKind {
    Coin,
    Token,
    Stake,
    Earn,
}

pub fn request_token_ids(token_ids: &[AssetId]) -> Vec<String> {
    token_ids.iter().filter_map(|asset_id| asset_id.token_id.clone()).collect()
}

pub fn chain_balances(coin: Vec<AssetBalance>, stake: Vec<AssetBalance>, tokens: Vec<AssetBalance>, earn: Vec<AssetBalance>) -> Vec<(BalanceKind, AssetBalance)> {
    [
        (BalanceKind::Coin, coin),
        (BalanceKind::Stake, stake),
        (BalanceKind::Token, tokens),
        (BalanceKind::Earn, earn),
    ]
    .into_iter()
    .flat_map(|(kind, balances)| balances.into_iter().map(move |balance| (kind, balance)))
    .collect()
}

pub fn balance_requests(accounts: &[Account], asset_ids: &[AssetId]) -> Vec<BalanceRequest> {
    accounts
        .iter()
        .filter_map(|account| {
            let chain = account.chain;
            let chain_asset_ids: Vec<&AssetId> = asset_ids.iter().filter(|asset_id| asset_id.chain == chain).collect();
            if chain_asset_ids.is_empty() {
                return None;
            }
            Some(BalanceRequest {
                chain,
                address: account.address.clone(),
                coin: chain_asset_ids.iter().any(|asset_id| asset_id.is_native()),
                token_ids: chain_asset_ids.into_iter().filter(|asset_id| asset_id.is_token()).cloned().collect(),
            })
        })
        .collect()
}

pub fn balance_updates(assets: &[Asset], balances: Vec<(BalanceKind, AssetBalance)>) -> Vec<GemBalanceUpdate> {
    let decimals: HashMap<AssetId, u32> = assets.iter().map(|asset| (asset.id.clone(), asset.decimals.max(0) as u32)).collect();
    balances
        .into_iter()
        .filter_map(|(kind, balance)| {
            let decimals = *decimals.get(&balance.asset_id)?;
            let value = |amount: &BigUint| GemBalanceValue {
                value: amount.to_string(),
                amount: BigNumberFormatter::value_as_f64(&amount.to_string(), decimals).unwrap_or_default(),
            };
            let update_type = match kind {
                BalanceKind::Coin => GemBalanceUpdateType::Coin {
                    available: value(&balance.balance.available),
                    reserved: value(&balance.balance.reserved),
                    pending_unconfirmed: value(&balance.balance.pending_unconfirmed),
                },
                BalanceKind::Token => GemBalanceUpdateType::Token {
                    available: value(&balance.balance.available),
                },
                BalanceKind::Stake => GemBalanceUpdateType::Stake {
                    staked: value(&balance.balance.staked),
                    pending: value(&balance.balance.pending),
                    rewards: value(&balance.balance.rewards),
                    locked: value(&balance.balance.locked),
                    frozen: value(&balance.balance.frozen),
                    metadata: balance.balance.metadata.clone(),
                },
                BalanceKind::Earn => GemBalanceUpdateType::Earn {
                    balance: value(&balance.balance.earn),
                },
            };
            Some(GemBalanceUpdate {
                asset_id: balance.asset_id,
                update_type,
                is_active: balance.is_active,
            })
        })
        .collect()
}

pub fn newly_enabled_asset_ids(requested: &[AssetId], enabled: &[AssetId]) -> Vec<AssetId> {
    missing(requested.iter().cloned(), enabled.iter().cloned())
}

pub fn unique_asset_ids(asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    unique(asset_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetType, Balance};

    fn account(chain: Chain, address: &str) -> Account {
        Account {
            chain,
            address: address.into(),
            derivation_path: "".into(),
            extended_public_key: None,
        }
    }

    #[test]
    fn test_balance_requests_match_tokens_by_typed_chain() {
        let sei = AssetId::from_chain(Chain::Sei);
        let sei_evm_token = AssetId::from_token(Chain::SeiEvm, "0xtoken");
        let ethereum_token = AssetId::from_token(Chain::Ethereum, "0xusdc");

        let requests = balance_requests(
            &[account(Chain::Sei, "sei-address"), account(Chain::Ethereum, "0xaddress")],
            &[sei.clone(), sei_evm_token, ethereum_token.clone()],
        );

        assert_eq!(
            requests,
            vec![
                BalanceRequest {
                    chain: Chain::Sei,
                    address: "sei-address".into(),
                    coin: true,
                    token_ids: vec![],
                },
                BalanceRequest {
                    chain: Chain::Ethereum,
                    address: "0xaddress".into(),
                    coin: false,
                    token_ids: vec![ethereum_token],
                },
            ]
        );
    }

    #[test]
    fn test_balance_updates_convert_with_asset_decimals_and_skip_unknown_assets() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let unknown = AssetId::from_token(Chain::Ethereum, "0xunknown");
        let asset = Asset::new(ethereum.clone(), "Ethereum".into(), "ETH".into(), 18, AssetType::NATIVE);

        let updates = balance_updates(
            &[asset],
            vec![
                (
                    BalanceKind::Coin,
                    AssetBalance::new_balance(ethereum.clone(), Balance::coin_balance(BigUint::from(1_500_000_000_000_000_000u64))),
                ),
                (BalanceKind::Token, AssetBalance::new(unknown, BigUint::from(1u64))),
            ],
        );

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].asset_id, ethereum);
        match &updates[0].update_type {
            GemBalanceUpdateType::Coin { available, .. } => {
                assert_eq!(available.value, "1500000000000000000");
                assert_eq!(available.amount, 1.5);
            }
            other => panic!("unexpected update {other:?}"),
        }
    }

    #[test]
    fn test_enable_asset_rules() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        assert_eq!(
            unique_asset_ids(vec![bitcoin.clone(), ethereum.clone(), bitcoin.clone()]),
            vec![bitcoin.clone(), ethereum.clone()]
        );
        assert_eq!(newly_enabled_asset_ids(&[bitcoin.clone(), ethereum.clone()], &[bitcoin]), vec![ethereum]);
    }

    #[test]
    fn test_request_token_ids_keeps_only_token_identifiers() {
        let token_ids = request_token_ids(&[AssetId::from_chain(Chain::Ethereum), AssetId::from_token(Chain::Ethereum, "0x1234")]);

        assert_eq!(token_ids, vec!["0x1234".to_string()]);
    }

    #[test]
    fn test_chain_balances_tags_every_balance_with_its_kind() {
        let balance = |asset_id: AssetId| AssetBalance::new(asset_id, BigUint::from(1u32));
        let coin = balance(AssetId::from_chain(Chain::Ethereum));
        let token = balance(AssetId::from_token(Chain::Ethereum, "0x1234"));

        let balances = chain_balances(vec![coin.clone()], Vec::new(), vec![token.clone()], Vec::new());

        assert_eq!(balances, vec![(BalanceKind::Coin, coin), (BalanceKind::Token, token)]);
    }
}
