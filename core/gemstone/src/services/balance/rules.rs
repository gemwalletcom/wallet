use std::collections::HashMap;

use crate::services::collections::{missing, unique};

use primitives::{Account, Asset, AssetBalance, AssetFiatValue, AssetId, BalanceCalculator, BalanceMetadata, Chain, TotalFiatValue};

use super::model::{GemAssetBalance, GemAssetConfiguration, GemBalanceRecord, GemBalanceUpdate, GemBalanceUpdateType};
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;
use crate::precision::GemValueStyle;
use crate::services::localization::GemLocalizedText;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::currency::Currency;

pub fn balance_amount(value: &BigUint, asset: &Asset) -> GemFormattedNumber {
    balance_amount_styled(value, asset, GemValueStyle::Auto)
}

pub fn available_balance_text(asset: Asset, balance: GemAssetBalance) -> GemLocalizedText {
    GemLocalizedText::Balance {
        amount: GemFormattedNumber::amount(BigNumberFormatter::f64_value(&balance.available, asset.decimals.unsigned_abs()), None, GemValueStyle::Auto),
    }
}

pub fn balance_amount_styled(value: &BigUint, asset: &Asset, style: GemValueStyle) -> GemFormattedNumber {
    let value = BigNumberFormatter::f64_value(value, asset.decimals.unsigned_abs());
    GemFormattedNumber::amount(value, Some(asset.symbol.clone()), style)
}

pub fn balance_resource_rows(metadata: Option<BalanceMetadata>) -> Vec<GemListRow> {
    let Some(metadata) = metadata else { return Vec::new() };
    let row = |title, available: u32, total: u32| GemListRow::Text {
        title,
        value: format!("{available} / {total}"),
    };
    vec![
        row(GemListRowTitle::Energy, metadata.energy_available, metadata.energy_total),
        row(GemListRowTitle::Bandwidth, metadata.bandwidth_available, metadata.bandwidth_total),
    ]
}

pub fn total_fiat_value(balances: &[AssetFiatValue]) -> TotalFiatValue {
    BalanceCalculator::total_fiat_value(balances)
}

/// The wallet header, finished: the total as it is, and the change beside it when there is one.
pub struct GemTotalHeader {
    pub total: GemFormattedNumber,
    pub pnl: Option<GemLocalizedText>,
    pub pnl_tone: GemValueTone,
}

pub fn total_header(total: &TotalFiatValue, currency: Currency) -> GemTotalHeader {
    let amount = GemFormattedNumber::signed_currency(total.pnl_amount, currency.clone(), GemCurrencyStyle::Fiat);
    GemTotalHeader {
        total: GemFormattedNumber::currency(total.value, currency, GemCurrencyStyle::Fiat),
        pnl_tone: amount.tone,
        pnl: shows_pnl(total).then(|| GemLocalizedText::Pnl {
            amount,
            percent: GemFormattedNumber::percentage(total.pnl_percentage, GemPercentageStyle::Unsigned),
        }),
    }
}

fn shows_pnl(total: &TotalFiatValue) -> bool {
    total.value > 0.0 && total.pnl_amount != 0.0
}

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

pub fn kind_balances<E>(kind: BalanceKind, balances: Result<Vec<AssetBalance>, E>) -> Result<Vec<(BalanceKind, AssetBalance)>, E> {
    balances.map(|balances| balances.into_iter().map(|balance| (kind, balance)).collect())
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

pub fn published_balances<E>(results: Vec<Result<Vec<(BalanceKind, AssetBalance)>, E>>) -> (Vec<(BalanceKind, AssetBalance)>, Option<E>) {
    let mut balances = Vec::new();
    let mut failure = None;
    for result in results {
        match result {
            Ok(chain) => balances.extend(chain),
            Err(error) => failure = failure.or(Some(error)),
        }
    }
    (balances, failure)
}

pub fn balance_updates(balances: Vec<(BalanceKind, AssetBalance)>) -> Vec<GemBalanceUpdate> {
    balances
        .into_iter()
        .map(|(kind, balance)| {
            let update_type = match kind {
                BalanceKind::Coin => GemBalanceUpdateType::Coin {
                    available: balance.balance.available,
                    frozen: balance.balance.frozen,
                    reserved: balance.balance.reserved,
                    pending_unconfirmed: balance.balance.pending_unconfirmed,
                },
                BalanceKind::Token => GemBalanceUpdateType::Token { available: balance.balance.available },
                BalanceKind::Stake => GemBalanceUpdateType::Stake {
                    staked: balance.balance.staked,
                    pending: balance.balance.pending,
                    rewards: balance.balance.rewards,
                    locked: balance.balance.locked,
                    frozen: balance.balance.frozen,
                    metadata: balance.balance.metadata,
                },
                BalanceKind::Earn => GemBalanceUpdateType::Earn { balance: balance.balance.earn },
            };
            GemBalanceUpdate {
                asset_id: balance.asset_id,
                update_type,
                is_active: balance.is_active,
            }
        })
        .collect()
}

pub fn changed_balances(stored: Vec<GemAssetBalance>, updates: Vec<GemBalanceUpdate>) -> Vec<GemAssetBalance> {
    let stored: HashMap<AssetId, GemAssetBalance> = stored.into_iter().map(|balance| (balance.asset_id.clone(), balance)).collect();
    let mut order = Vec::new();
    let mut applied: HashMap<AssetId, GemAssetBalance> = HashMap::new();
    for update in updates {
        let balance = applied.entry(update.asset_id.clone()).or_insert_with(|| {
            order.push(update.asset_id.clone());
            stored.get(&update.asset_id).cloned().unwrap_or_else(|| GemAssetBalance::zero(update.asset_id.clone()))
        });
        *balance = balance.applying(&update);
    }
    order.into_iter().filter_map(|asset_id| applied.remove(&asset_id)).filter(|balance| stored.get(&balance.asset_id) != Some(balance)).collect()
}

pub fn balance_records(balances: Vec<GemAssetBalance>, assets: &[Asset]) -> Vec<GemBalanceRecord> {
    let decimals: HashMap<AssetId, u32> = assets.iter().map(|asset| (asset.id.clone(), asset.decimals.max(0) as u32)).collect();
    balances.into_iter().filter_map(|balance| Some(GemBalanceRecord::new(balance.clone(), *decimals.get(&balance.asset_id)?))).collect()
}

pub fn enabled_configuration(enabled: bool) -> GemAssetConfiguration {
    GemAssetConfiguration {
        is_enabled: Some(enabled),
        is_pinned: (!enabled).then_some(false),
    }
}

pub fn pinned_configuration(pinned: bool) -> GemAssetConfiguration {
    GemAssetConfiguration { is_enabled: None, is_pinned: Some(pinned) }
}

pub fn missing_asset_ids(requested: &[AssetId], stored: &[AssetId]) -> Vec<AssetId> {
    missing(requested.iter().cloned(), stored.iter().cloned())
}

pub fn unique_asset_ids(asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    unique(asset_ids)
}

pub fn exclude_native_mirrors(asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    asset_ids.into_iter().filter(|asset_id| !asset_id.is_native_mirror()).collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_the_available_balance_reads_as_one_sentence_without_a_symbol() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let balance = GemAssetBalance::mock_with_available(2_000_000_000_000_000_000);

        let GemLocalizedText::Balance { amount } = available_balance_text(asset.clone(), balance) else {
            panic!("a balance sentence carries an amount");
        };

        assert_eq!(amount.value, 2.0);
        assert_eq!(amount.unit, crate::formatted_number::GemNumberUnit::Plain, "the swap input names the asset elsewhere");
    }

    #[test]
    fn test_the_header_shows_the_total_as_it_is_and_the_change_only_when_there_is_one() {
        let header = total_header(
            &TotalFiatValue {
                value: 1_250.0,
                pnl_amount: -25.0,
                pnl_percentage: 2.0,
            },
            Currency::USD,
        );
        assert_eq!(header.total.value, 1_250.0);
        assert_eq!(header.pnl_tone, GemValueTone::Negative, "the change takes its tone from the amount, not the percent");
        assert!(matches!(header.pnl, Some(GemLocalizedText::Pnl { amount, percent }) if amount.value == -25.0 && percent.value == 2.0));

        let empty = total_header(
            &TotalFiatValue {
                value: 0.0,
                pnl_amount: 0.0,
                pnl_percentage: 0.0,
            },
            Currency::USD,
        );
        assert_eq!(empty.pnl, None, "an empty wallet has no change to show");

        let negative = total_header(
            &TotalFiatValue {
                value: -5.0,
                pnl_amount: 0.0,
                pnl_percentage: 0.0,
            },
            Currency::USD,
        );
        assert_eq!(negative.total.value, -5.0, "a total that is negative reads as it is, never as zero");
    }

    #[test]
    fn test_a_tron_resource_reads_available_over_total() {
        assert!(balance_resource_rows(None).is_empty(), "a chain with no resources has no rows");
        let rows = balance_resource_rows(Some(BalanceMetadata {
            votes: 0,
            energy_available: 100,
            energy_total: 250,
            bandwidth_available: 5,
            bandwidth_total: 600,
        }));
        assert_eq!(
            rows,
            vec![
                GemListRow::Text {
                    title: GemListRowTitle::Energy,
                    value: "100 / 250".to_string()
                },
                GemListRow::Text {
                    title: GemListRowTitle::Bandwidth,
                    value: "5 / 600".to_string()
                },
            ]
        );
    }

    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn test_changed_balances_keeps_only_what_differs_from_the_stored_row() {
        let stored = GemAssetBalance::mock_with_available(10);

        assert!(
            changed_balances(vec![stored.clone()], vec![GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(10u32) })]).is_empty(),
            "same value and state is not a change"
        );
        assert_eq!(
            changed_balances(vec![stored.clone()], vec![GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(11u32) })]).len(),
            1,
            "a new value is"
        );
        assert_eq!(
            changed_balances(
                vec![stored.clone()],
                vec![GemBalanceUpdate {
                    is_active: false,
                    ..GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(10u32) })
                }]
            )
            .len(),
            1,
            "so is an activation change alone"
        );
        assert_eq!(
            changed_balances(vec![], vec![GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(10u32) })]).len(),
            1,
            "a balance with no stored row is always written"
        );

        let stake = GemBalanceUpdate::mock(GemBalanceUpdateType::Stake {
            staked: BigUint::ZERO,
            pending: BigUint::ZERO,
            rewards: BigUint::ZERO,
            locked: BigUint::ZERO,
            frozen: BigUint::ZERO,
            metadata: None,
        });
        assert!(
            changed_balances(vec![stored.clone()], vec![stake.clone()]).is_empty(),
            "a stake update leaves the coin's available alone and compares its own fields"
        );

        let folded = changed_balances(vec![stored], vec![GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(11u32) }), stake]);
        assert_eq!(folded.len(), 1, "two updates for one asset fold into one row");
        assert_eq!(folded[0].available, BigUint::from(11u32));
    }
    use primitives::{AssetType, Balance, asset_constants::ARC_USDC_ASSET_ID};

    #[test]
    fn test_pnl_shows_only_for_a_funded_wallet_that_moved() {
        assert!(shows_pnl(&TotalFiatValue {
            value: 20.0,
            pnl_amount: 1.0,
            pnl_percentage: 0.0,
        }));
        assert!(!shows_pnl(&TotalFiatValue {
            value: 20.0,
            pnl_amount: 0.0,
            pnl_percentage: 0.0,
        }));
        assert!(!shows_pnl(&TotalFiatValue {
            value: 0.0,
            pnl_amount: 1.0,
            pnl_percentage: 0.0,
        }));
    }

    #[test]
    fn test_balance_requests_match_tokens_by_typed_chain() {
        let sei = AssetId::from_chain(Chain::Sei);
        let sei_evm_token = AssetId::from_token(Chain::SeiEvm, "0xtoken");
        let ethereum_token = AssetId::from_token(Chain::Ethereum, "0xusdc");

        let requests = balance_requests(&[Account::mock(Chain::Sei, "sei-address"), Account::mock(Chain::Ethereum, "0xaddress")], &[sei.clone(), sei_evm_token, ethereum_token.clone()]);

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
    fn test_balance_records_convert_with_asset_decimals_and_skip_unknown_assets() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let unknown = AssetId::from_token(Chain::Ethereum, "0xunknown");
        let asset = Asset::new(ethereum.clone(), "Ethereum".into(), "ETH".into(), 18, AssetType::NATIVE);
        let updates = balance_updates(vec![
            (BalanceKind::Coin, AssetBalance::new_balance(ethereum.clone(), Balance::coin_balance(BigUint::from(1_500_000_000_000_000_000u64)))),
            (BalanceKind::Token, AssetBalance::new(unknown, BigUint::from(1u64))),
        ]);

        let records = balance_records(changed_balances(vec![], updates), &[asset]);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].asset_id, ethereum);
        assert_eq!(records[0].available.value, BigUint::from(1_500_000_000_000_000_000u64));
        assert_eq!(records[0].available.amount, 1.5);
        assert_eq!(records[0].staked.amount, 0.0);
    }

    #[test]
    fn test_enable_asset_rules() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        assert_eq!(unique_asset_ids(vec![bitcoin.clone(), ethereum.clone(), bitcoin.clone()]), vec![bitcoin.clone(), ethereum.clone()]);
        assert_eq!(missing_asset_ids(&[bitcoin.clone(), ethereum.clone()], &[bitcoin]), vec![ethereum]);
    }

    #[test]
    fn test_exclude_native_mirrors() {
        let arc = AssetId::from_chain(Chain::Arc);

        assert_eq!(exclude_native_mirrors(vec![arc.clone(), ARC_USDC_ASSET_ID.clone()]), vec![arc]);
    }

    #[test]
    fn test_request_token_ids_keeps_only_token_identifiers() {
        let token_ids = request_token_ids(&[AssetId::from_chain(Chain::Ethereum), AssetId::from_token(Chain::Ethereum, "0x1234")]);

        assert_eq!(token_ids, vec!["0x1234".to_string()]);
    }

    #[test]
    fn test_a_component_that_fails_never_holds_back_the_components_that_answered() {
        let ethereum = AssetId::from_chain(Chain::Ethereum);
        let token = AssetId::from_token(Chain::Ethereum, "0x1234");
        let (balances, failure) = published_balances(vec![
            kind_balances(BalanceKind::Coin, Ok(vec![AssetBalance::new(ethereum.clone(), BigUint::from(1u32))])),
            kind_balances(BalanceKind::Stake, Err("staking is offline")),
            kind_balances(BalanceKind::Token, Ok(vec![AssetBalance::new(token.clone(), BigUint::from(1u32))])),
            kind_balances(BalanceKind::Earn, Err("earn is offline")),
        ]);

        assert_eq!(
            balances,
            vec![(BalanceKind::Coin, AssetBalance::new(ethereum, BigUint::from(1u32))), (BalanceKind::Token, AssetBalance::new(token, BigUint::from(1u32)))],
            "the coin and token balances of a network are published when its staking request fails"
        );
        assert_eq!(failure, Some("staking is offline"), "the caller hears about the first failure in request order");

        let (balances, failure) = published_balances::<&str>(vec![Ok(vec![(BalanceKind::Coin, AssetBalance::new(AssetId::from_chain(Chain::Bitcoin), BigUint::from(1u32)))])]);
        assert_eq!(balances.len(), 1);
        assert_eq!(failure, None);
    }

    #[test]
    fn test_kind_balances_tags_every_balance_with_its_kind() {
        let token = AssetBalance::new(AssetId::from_token(Chain::Ethereum, "0x1234"), BigUint::from(1u32));

        assert_eq!(kind_balances::<&str>(BalanceKind::Token, Ok(vec![token.clone()])), Ok(vec![(BalanceKind::Token, token)]));
        assert_eq!(kind_balances::<&str>(BalanceKind::Stake, Err("offline")), Err("offline"));
    }
}
