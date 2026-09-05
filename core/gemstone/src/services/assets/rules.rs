use std::str::FromStr;

use primitives::{
    Asset, AssetBasic, AssetId, AssetMetaData, AssetPrice, AssetProperties, AssetScore, BannerEvent, Chain, ConfigVersions, StakeChain, VerificationStatus, Wallet, WalletType,
};

use super::model::{
    AssetList, GemAssetAction, GemAssetDetailsState, GemAssetEmptyAction, GemAssetNetworkDestination, GemHeaderButton, GemHeaderButtonKind, GemSelectAssetFlow, GemSelectAssetType,
    GemSelectRowAction, GemWalletSearchLimits,
};
use crate::config::search_config::{ASSETS_INITIAL_LIMIT, ASSETS_SEARCH_LIMIT, NFTS_PREVIEW_LIMIT, PERPETUALS_PREVIEW_LIMIT, RESULTS_LIMIT};
use crate::models::custom_types::GemBigUint;
use crate::services::balance::GemAssetBalance;

use crate::models::asset::{wallet_asset_is_enabled, wallet_default_assets};
use crate::services::collections::{missing, missing_by, unique};

pub fn asset_list_versions(versions: &ConfigVersions) -> [(AssetList, i32); 3] {
    [
        (AssetList::Buy, versions.fiat_on_ramp_assets),
        (AssetList::Sell, versions.fiat_off_ramp_assets),
        (AssetList::Swap, versions.swap_assets),
    ]
}

pub fn is_asset_list_outdated(stored_version: Option<&str>, remote_version: i32) -> bool {
    stored_version != Some(remote_version.to_string().as_str())
}

pub fn asset_ids(ids: &[String]) -> Vec<AssetId> {
    ids.iter().filter_map(|id| AssetId::new(id)).collect()
}

pub fn swappable_chain_asset_ids() -> Vec<AssetId> {
    Chain::all().into_iter().filter(Chain::is_swap_supported).map(AssetId::from_chain).collect()
}

pub fn token_search_chains(chains: &[Chain]) -> Vec<Chain> {
    if chains.is_empty() { Chain::all() } else { chains.to_vec() }
}

pub fn missing_asset_ids(requested: Vec<AssetId>, existing: Vec<AssetId>) -> Vec<AssetId> {
    missing(requested, existing)
}

pub fn asset_prices(assets: &[AssetBasic]) -> Vec<AssetPrice> {
    assets
        .iter()
        .filter_map(|asset| {
            asset
                .price
                .as_ref()
                .map(|price| AssetPrice::new(asset.asset.id.clone(), price.price, price.price_change_percentage_24h, price.updated_at))
        })
        .collect()
}

pub fn default_asset_basic(asset: Asset) -> AssetBasic {
    let asset_id = asset.id.clone();
    AssetBasic::new(asset, AssetProperties::default(asset_id.clone()), AssetScore::new(asset_id.default_rank()))
}

pub fn default_assets() -> Vec<AssetBasic> {
    Chain::all()
        .into_iter()
        .flat_map(|chain| std::iter::once(Asset::from_chain(chain)).chain(wallet_default_assets(chain)))
        .map(default_asset_basic)
        .collect()
}

pub fn missing_assets(assets: Vec<AssetBasic>, existing: Vec<AssetId>) -> Vec<AssetBasic> {
    missing_by(assets, existing, |asset| asset.asset.id.clone())
}

pub fn stakeable_asset_ids() -> Vec<AssetId> {
    Chain::all().into_iter().filter(Chain::is_stake_supported).map(AssetId::from_chain).collect()
}

pub fn default_token_chain(chains: &[Chain]) -> Option<Chain> {
    chains.iter().find(|chain| **chain == Chain::Ethereum).or(chains.first()).copied()
}

pub fn token_chains(wallet: &Wallet) -> Vec<Chain> {
    let mut chains = unique(wallet.accounts.iter().map(|account| account.chain).filter(|chain| chain.default_asset_type().is_some()));
    chains.sort_by_key(|chain| std::cmp::Reverse(AssetId::from_chain(*chain).default_rank()));
    chains
}

pub fn popular_asset_ids() -> Vec<AssetId> {
    [Chain::Bitcoin, Chain::Ethereum, Chain::Solana].into_iter().map(AssetId::from_chain).collect()
}

pub fn can_open(wallet: &Wallet, asset_id: &AssetId) -> bool {
    (asset_id.is_token() || asset_id.chain.has_native_asset()) && wallet.account(asset_id.chain).is_some()
}

pub fn default_balances(wallet: &Wallet) -> (Vec<AssetId>, Vec<AssetId>) {
    unique(wallet.accounts.iter().flat_map(|account| {
        let chain = account.chain;
        let native = (chain.rank() >= 0).then(|| AssetId::from_chain(chain));
        native.into_iter().chain(wallet_default_assets(chain).into_iter().map(|asset| asset.id))
    }))
    .into_iter()
    .partition(|asset_id| wallet_asset_is_enabled(asset_id.clone(), wallet.wallet_type.clone()))
}

pub fn icon_asset_id(asset_id: &AssetId) -> AssetId {
    perpetual_coin(asset_id)
        .and_then(|coin| Chain::all().into_iter().find(|chain| Asset::from_chain(*chain).symbol == coin))
        .map(AssetId::from_chain)
        .unwrap_or_else(|| asset_id.clone())
}

fn perpetual_coin(asset_id: &AssetId) -> Option<String> {
    let ids = AssetId::decode_token_id(asset_id.token_id.as_deref()?);
    (asset_id.chain == Chain::HyperCore && ids.first().is_some_and(|kind| kind == "perpetual")).then(|| ids.get(1).cloned())?
}

pub fn network_destination(asset_id: &AssetId) -> Option<GemAssetNetworkDestination> {
    let chain = asset_id.chain;
    if asset_id.is_token() && chain.has_native_asset() {
        return Some(GemAssetNetworkDestination::Asset { asset: Asset::from_chain(chain) });
    }
    chain.default_asset_type().is_some().then_some(GemAssetNetworkDestination::Assets { chain })
}

pub fn verification_status(asset: &Asset, rank: i32) -> Option<VerificationStatus> {
    if asset.id.is_native() {
        return None;
    }
    match VerificationStatus::from_rank(rank) {
        VerificationStatus::Unverified => Some(VerificationStatus::Unverified),
        VerificationStatus::Verified | VerificationStatus::Suspicious => None,
    }
}

pub fn select_asset_flow(select_type: GemSelectAssetType) -> GemSelectAssetFlow {
    let flow = |row_action: GemSelectRowAction, action: Option<GemAssetAction>| GemSelectAssetFlow {
        row_action,
        action,
        enables_price_alert: false,
        network_search: false,
        chain_filter: false,
        recents: false,
        popular_section: false,
        balance_filter: false,
        add_custom_token: false,
        deposit_asset_display: false,
    };
    match select_type {
        GemSelectAssetType::Send => GemSelectAssetFlow {
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Send))
        },
        GemSelectAssetType::Receive => GemSelectAssetFlow {
            network_search: true,
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Receive))
        },
        GemSelectAssetType::ReceiveCollection => GemSelectAssetFlow {
            network_search: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Receive))
        },
        GemSelectAssetType::Buy => GemSelectAssetFlow {
            network_search: true,
            chain_filter: true,
            recents: true,
            popular_section: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Buy))
        },
        GemSelectAssetType::SwapPay => GemSelectAssetFlow {
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Select, Some(GemAssetAction::SwapPay))
        },
        GemSelectAssetType::SwapReceive => GemSelectAssetFlow {
            network_search: true,
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Select, Some(GemAssetAction::SwapReceive))
        },
        GemSelectAssetType::Manage => GemSelectAssetFlow {
            network_search: true,
            chain_filter: true,
            balance_filter: true,
            add_custom_token: true,
            ..flow(GemSelectRowAction::Toggle, None)
        },
        GemSelectAssetType::PriceAlert => GemSelectAssetFlow {
            enables_price_alert: true,
            network_search: true,
            chain_filter: true,
            popular_section: true,
            ..flow(GemSelectRowAction::Select, None)
        },
        GemSelectAssetType::Deposit => flow(GemSelectRowAction::Navigate, None),
        GemSelectAssetType::Withdraw => GemSelectAssetFlow {
            deposit_asset_display: true,
            ..flow(GemSelectRowAction::Navigate, None)
        },
        GemSelectAssetType::WalletSearch => GemSelectAssetFlow {
            network_search: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Open))
        },
        GemSelectAssetType::WalletSearchResults => flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Open)),
    }
}

pub fn wallet_search_limits(query: &str) -> GemWalletSearchLimits {
    let assets = match query.trim().is_empty() {
        true => ASSETS_INITIAL_LIMIT,
        false => ASSETS_SEARCH_LIMIT,
    };
    GemWalletSearchLimits {
        assets,
        fetch: assets + 1,
        perpetuals: PERPETUALS_PREVIEW_LIMIT,
        nfts: NFTS_PREVIEW_LIMIT,
        results: RESULTS_LIMIT,
    }
}

pub fn details_state(
    wallet_type: WalletType,
    chain: Chain,
    metadata: &AssetMetaData,
    balance: &GemAssetBalance,
    banner_events: &[BannerEvent],
    has_price: bool,
    price_alerts_count: u32,
) -> GemAssetDetailsState {
    let is_view_only = wallet_type == WalletType::View;
    let buttons_enabled = !banner_events
        .iter()
        .any(|event| matches!(event, BannerEvent::ActivateAsset | BannerEvent::AccountBlockedMultiSignature));
    let button = |kind: GemHeaderButtonKind, shows: bool| {
        (shows && !is_view_only).then_some(GemHeaderButton {
            kind,
            is_enabled: buttons_enabled,
        })
    };
    GemAssetDetailsState {
        is_view_only,
        header_buttons: [
            button(GemHeaderButtonKind::Send, true),
            button(GemHeaderButtonKind::Receive, true),
            button(GemHeaderButtonKind::Buy, metadata.is_buy_enabled),
            button(GemHeaderButtonKind::Swap, metadata.is_swap_enabled),
        ]
        .into_iter()
        .flatten()
        .collect(),
        shows_banners: !is_view_only,
        shows_manage: !metadata.is_balance_enabled,
        shows_resources: StakeChain::from_str(chain.as_ref()).is_ok_and(|stake_chain| stake_chain.get_uses_freeze()),
        shows_price_alerts: price_alerts_count > 0 && has_price,
        shows_earn: metadata.is_earn_enabled && !is_view_only && balance.earn == GemBigUint::ZERO,
        empty_transactions_action: if metadata.is_buy_enabled {
            Some(GemAssetEmptyAction::Buy)
        } else if metadata.is_swap_enabled {
            Some(GemAssetEmptyAction::Swap)
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_each_select_flow_decides_its_row_action_and_recent_activity() {
        let row = |select_type: GemSelectAssetType| (select_type.flow().row_action, select_type.flow().action);
        assert_eq!(row(GemSelectAssetType::Send), (GemSelectRowAction::Navigate, Some(GemAssetAction::Send)));
        assert_eq!(row(GemSelectAssetType::Receive), (GemSelectRowAction::Navigate, Some(GemAssetAction::Receive)));
        assert_eq!(row(GemSelectAssetType::ReceiveCollection), (GemSelectRowAction::Navigate, Some(GemAssetAction::Receive)));
        assert_eq!(row(GemSelectAssetType::Buy), (GemSelectRowAction::Navigate, Some(GemAssetAction::Buy)));
        assert_eq!(row(GemSelectAssetType::SwapPay), (GemSelectRowAction::Select, Some(GemAssetAction::SwapPay)));
        assert_eq!(row(GemSelectAssetType::SwapReceive), (GemSelectRowAction::Select, Some(GemAssetAction::SwapReceive)));
        assert_eq!(row(GemSelectAssetType::Manage), (GemSelectRowAction::Toggle, None));
        assert_eq!(row(GemSelectAssetType::PriceAlert), (GemSelectRowAction::Select, None));
        assert_eq!(row(GemSelectAssetType::Deposit), (GemSelectRowAction::Navigate, None));
        assert_eq!(row(GemSelectAssetType::Withdraw), (GemSelectRowAction::Navigate, None));
        assert_eq!(row(GemSelectAssetType::WalletSearch), (GemSelectRowAction::Navigate, Some(GemAssetAction::Open)));
        assert_eq!(row(GemSelectAssetType::WalletSearchResults), (GemSelectRowAction::Navigate, Some(GemAssetAction::Open)));
    }

    #[test]
    fn test_each_select_flow_enables_only_its_capabilities() {
        let enabled = |select_type: GemSelectAssetType| {
            let flow = select_type.flow();
            [
                ("network_search", flow.network_search),
                ("chain_filter", flow.chain_filter),
                ("recents", flow.recents),
                ("popular_section", flow.popular_section),
                ("balance_filter", flow.balance_filter),
                ("add_custom_token", flow.add_custom_token),
                ("deposit_asset_display", flow.deposit_asset_display),
                ("enables_price_alert", flow.enables_price_alert),
            ]
            .into_iter()
            .filter_map(|(name, on)| on.then_some(name))
            .collect::<Vec<_>>()
        };
        assert_eq!(enabled(GemSelectAssetType::Send), ["chain_filter", "recents"]);
        assert_eq!(enabled(GemSelectAssetType::Receive), ["network_search", "chain_filter", "recents"]);
        assert_eq!(enabled(GemSelectAssetType::ReceiveCollection), ["network_search", "recents"]);
        assert_eq!(enabled(GemSelectAssetType::Buy), ["network_search", "chain_filter", "recents", "popular_section"]);
        assert_eq!(enabled(GemSelectAssetType::SwapPay), ["chain_filter", "recents"]);
        assert_eq!(enabled(GemSelectAssetType::SwapReceive), ["network_search", "chain_filter", "recents"]);
        assert_eq!(
            enabled(GemSelectAssetType::Manage),
            ["network_search", "chain_filter", "balance_filter", "add_custom_token"]
        );
        assert_eq!(
            enabled(GemSelectAssetType::PriceAlert),
            ["network_search", "chain_filter", "popular_section", "enables_price_alert"]
        );
        assert!(enabled(GemSelectAssetType::Deposit).is_empty());
        assert_eq!(enabled(GemSelectAssetType::Withdraw), ["deposit_asset_display"]);
        assert_eq!(enabled(GemSelectAssetType::WalletSearch), ["network_search", "recents"]);
        assert!(enabled(GemSelectAssetType::WalletSearchResults).is_empty());
    }

    #[test]
    fn test_wallet_search_limits_widen_while_searching_and_fetch_one_more_than_shown() {
        let initial = wallet_search_limits("  ");
        let searching = wallet_search_limits("btc");

        assert_eq!((initial.assets, initial.fetch), (12, 13));
        assert_eq!((searching.assets, searching.fetch), (25, 26));
        assert_eq!((initial.perpetuals, initial.nfts, initial.results), (3, 3, 100));
        assert_eq!((searching.perpetuals, searching.nfts, searching.results), (3, 3, 100));
    }

    #[test]
    fn test_icon_asset_id_borrows_the_underlying_chain_icon_for_a_perpetual() {
        let bitcoin_perpetual = AssetId::from(Chain::HyperCore, Some(AssetId::sub_token_id(&["perpetual".to_string(), "BTC".to_string()])));
        assert_eq!(icon_asset_id(&bitcoin_perpetual), AssetId::from_chain(Chain::Bitcoin));
        let unknown_perpetual = AssetId::from(Chain::HyperCore, Some(AssetId::sub_token_id(&["perpetual".to_string(), "PUMP".to_string()])));
        assert_eq!(icon_asset_id(&unknown_perpetual), unknown_perpetual);
        let token = Asset::mock_ethereum_usdc().id;
        assert_eq!(icon_asset_id(&token), token);
    }

    #[test]
    fn test_network_destination_opens_the_native_asset_or_the_chain_list() {
        let ethereum = Chain::Ethereum;
        assert_eq!(
            network_destination(&Asset::mock_ethereum_usdc().id),
            Some(GemAssetNetworkDestination::Asset {
                asset: Asset::from_chain(ethereum)
            })
        );
        assert_eq!(
            network_destination(&AssetId::from_token(Chain::Tempo, "0x1")),
            Some(GemAssetNetworkDestination::Assets { chain: Chain::Tempo })
        );
        assert_eq!(
            network_destination(&AssetId::from_chain(ethereum)),
            Some(GemAssetNetworkDestination::Assets { chain: ethereum })
        );
        assert_eq!(network_destination(&AssetId::from_chain(Chain::Bitcoin)), None);
    }

    #[test]
    fn test_verification_status_rows_unverified_tokens_only() {
        let token = Asset::mock_ethereum_usdc();
        assert_eq!(verification_status(&token, 10), Some(VerificationStatus::Unverified));
        assert_eq!(verification_status(&token, 3), None);
        assert_eq!(verification_status(&token, 20), None);
        assert_eq!(verification_status(&Asset::mock(), 10), None);
    }

    #[test]
    fn test_default_token_chain_prefers_ethereum_then_first() {
        assert_eq!(default_token_chain(&[Chain::Solana, Chain::Ethereum]), Some(Chain::Ethereum));
        assert_eq!(default_token_chain(&[Chain::Solana, Chain::Tron]), Some(Chain::Solana));
        assert_eq!(default_token_chain(&[]), None);
    }

    #[test]
    fn test_token_chains_keeps_token_networks_by_rank() {
        let multicoin = wallet(WalletType::Multicoin, &[Chain::Bitcoin, Chain::Doge, Chain::Near, Chain::Xrp, Chain::Ethereum, Chain::Near]);
        assert_eq!(token_chains(&multicoin), vec![Chain::Ethereum, Chain::Xrp, Chain::Near]);
        assert!(token_chains(&wallet(WalletType::Single, &[Chain::Bitcoin])).is_empty());
    }

    #[test]
    fn test_popular_asset_ids_are_distinct_native_assets() {
        let ids = popular_asset_ids();

        assert!(!ids.is_empty());
        assert!(ids.iter().all(|id| id.is_native() && id.chain.has_native_asset()));
        assert_eq!(unique(ids.clone()).len(), ids.len());
        assert_eq!(ids.first(), Some(&AssetId::from_chain(Chain::Bitcoin)));
    }

    #[test]
    fn test_can_open_requires_account_and_native_asset() {
        let wallet = wallet(WalletType::Multicoin, &[Chain::Ethereum, Chain::Tempo]);
        assert!(can_open(&wallet, &AssetId::from_chain(Chain::Ethereum)));
        assert!(can_open(
            &wallet,
            &AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()))
        ));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Bitcoin)));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Tempo)));
        assert!(can_open(
            &wallet,
            &AssetId::from(Chain::Tempo, Some("0x20c000000000000000000000c48d6a3bd5b7b0c2".to_string()))
        ));
    }

    #[test]
    fn test_default_assets_and_missing() {
        let assets = default_assets();
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let tron_usdt = wallet_default_assets(Chain::Tron)[0].id.clone();
        assert!(assets.iter().any(|asset| asset.asset.id == bitcoin));
        assert!(assets.iter().any(|asset| asset.asset.id == tron_usdt));

        let missing = missing_assets(assets.clone(), vec![bitcoin.clone()]);
        assert_eq!(missing.len(), assets.len() - 1);
        assert!(!missing.iter().any(|asset| asset.asset.id == bitcoin));
        assert!(stakeable_asset_ids().contains(&AssetId::from_chain(Chain::Cosmos)));
        assert!(!stakeable_asset_ids().contains(&bitcoin));
    }
    use primitives::{Account, Chain, WalletType};

    fn wallet(wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            wallet_type,
            ..Wallet::mock_with_accounts(Account::mock_chains(chains, "address"))
        }
    }

    #[test]
    fn test_default_asset_basic_uses_default_rank_and_properties() {
        let native = default_asset_basic(Asset::from_chain(Chain::Ethereum));
        let token = default_asset_basic(Asset::new(
            AssetId::from(Chain::Ethereum, Some("0x0000000000000000000000000000000000000001".to_string())),
            String::new(),
            String::new(),
            18,
            primitives::AssetType::ERC20,
        ));

        assert_eq!(native.score.rank, Chain::Ethereum.rank());
        assert!(native.score.rank > token.score.rank);
        assert!(native.properties.is_enabled);
        assert!(native.price.is_none());
    }

    #[test]
    fn test_default_balances_by_wallet_type() {
        let (enabled, disabled) = default_balances(&wallet(WalletType::Multicoin, &[Chain::Cosmos, Chain::Ethereum, Chain::Tron]));
        assert!(disabled.contains(&AssetId::from_chain(Chain::Cosmos)));
        assert!(enabled.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(wallet_default_assets(Chain::Tron).iter().all(|asset| enabled.contains(&asset.id)));

        let (enabled, disabled) = default_balances(&wallet(WalletType::Single, &[Chain::Cosmos]));
        assert_eq!(enabled, vec![AssetId::from_chain(Chain::Cosmos)]);
        assert!(disabled.is_empty());

        let (enabled, _) = default_balances(&wallet(WalletType::Single, &[Chain::Tempo]));
        assert!(!enabled.contains(&AssetId::from_chain(Chain::Tempo)));
        assert!(wallet_default_assets(Chain::Tempo).iter().all(|asset| enabled.contains(&asset.id)));
    }

    #[test]
    fn test_missing_asset_ids_drops_known_and_duplicate_ids() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        let missing = missing_asset_ids(vec![bitcoin.clone(), ethereum.clone(), ethereum.clone()], vec![bitcoin]);

        assert_eq!(missing, vec![ethereum]);
    }

    #[test]
    fn test_asset_list_is_outdated_only_when_the_stored_version_differs() {
        assert!(is_asset_list_outdated(None, 7));
        assert!(is_asset_list_outdated(Some("6"), 7));
        assert!(!is_asset_list_outdated(Some("7"), 7));
    }

    #[test]
    fn test_asset_ids_skips_unparsable_identifiers() {
        let ids = asset_ids(&["bitcoin".to_string(), String::new(), "ethereum_0x1234".to_string()]);

        assert_eq!(ids, vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_token(Chain::Ethereum, "0x1234")]);
    }

    #[test]
    fn test_swappable_chain_asset_ids_only_lists_swap_supported_chains() {
        let asset_ids = swappable_chain_asset_ids();

        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(asset_ids.iter().all(|asset_id| asset_id.chain.is_swap_supported()));
    }

    #[test]
    fn test_token_search_chains_defaults_to_every_chain() {
        assert_eq!(token_search_chains(&[Chain::Ethereum]), vec![Chain::Ethereum]);
        assert_eq!(token_search_chains(&[]), Chain::all());
    }

    fn metadata(is_balance_enabled: bool, is_buy_enabled: bool, is_swap_enabled: bool, is_earn_enabled: bool) -> AssetMetaData {
        AssetMetaData {
            is_enabled: true,
            is_balance_enabled,
            is_buy_enabled,
            is_sell_enabled: false,
            is_swap_enabled,
            is_stake_enabled: false,
            is_earn_enabled,
            is_pinned: false,
            is_active: true,
            staking_apr: None,
            earn_apr: None,
            rank_score: 0,
        }
    }

    fn state(wallet_type: WalletType, chain: Chain, metadata: &AssetMetaData, banner_events: &[BannerEvent]) -> GemAssetDetailsState {
        details_state(wallet_type, chain, metadata, &GemAssetBalance::mock(), banner_events, true, 0)
    }

    fn kinds(state: &GemAssetDetailsState) -> Vec<GemHeaderButtonKind> {
        state.header_buttons.iter().map(|button| button.kind).collect()
    }

    #[test]
    fn test_details_state_shows_the_header_buttons_the_metadata_allows() {
        let all = state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, true, true, false), &[]);
        assert_eq!(
            kinds(&all),
            vec![GemHeaderButtonKind::Send, GemHeaderButtonKind::Receive, GemHeaderButtonKind::Buy, GemHeaderButtonKind::Swap]
        );
        assert!(all.header_buttons.iter().all(|button| button.is_enabled));

        let transfer_only = state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, false, false, false), &[]);
        assert_eq!(kinds(&transfer_only), vec![GemHeaderButtonKind::Send, GemHeaderButtonKind::Receive]);
    }

    #[test]
    fn test_details_state_disables_the_buttons_behind_an_activation_or_multi_signature_banner() {
        for event in [BannerEvent::ActivateAsset, BannerEvent::AccountBlockedMultiSignature] {
            let state = state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, true, true, false), &[BannerEvent::Stake, event]);
            assert!(state.header_buttons.iter().all(|button| !button.is_enabled), "{event:?}");
        }
        let stake_only = state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, true, true, false), &[BannerEvent::Stake]);
        assert!(stake_only.header_buttons.iter().all(|button| button.is_enabled));
    }

    #[test]
    fn test_details_state_hides_buttons_banners_and_earn_from_a_view_only_wallet() {
        let state = state(WalletType::View, Chain::Ethereum, &metadata(true, true, true, true), &[]);

        assert!(state.is_view_only);
        assert!(state.header_buttons.is_empty());
        assert!(!state.shows_banners);
        assert!(!state.shows_earn);
        assert_eq!(state.empty_transactions_action, Some(GemAssetEmptyAction::Buy));
    }

    #[test]
    fn test_details_state_offers_manage_until_the_balance_is_enabled() {
        assert!(state(WalletType::Multicoin, Chain::Ethereum, &metadata(false, false, false, false), &[]).shows_manage);
        assert!(!state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, false, false, false), &[]).shows_manage);

        let disabled_asset = AssetMetaData {
            is_enabled: false,
            ..metadata(false, false, false, false)
        };
        assert!(state(WalletType::Multicoin, Chain::Ethereum, &disabled_asset, &[]).shows_manage);
    }

    #[test]
    fn test_details_state_shows_resources_only_where_staking_freezes() {
        assert!(state(WalletType::Multicoin, Chain::Tron, &metadata(true, false, false, false), &[]).shows_resources);
        assert!(!state(WalletType::Multicoin, Chain::Cosmos, &metadata(true, false, false, false), &[]).shows_resources);
        assert!(!state(WalletType::Multicoin, Chain::Bitcoin, &metadata(true, false, false, false), &[]).shows_resources);
    }

    #[test]
    fn test_details_state_shows_price_alerts_only_with_alerts_and_a_price() {
        let plain = metadata(true, false, false, false);
        let balance = GemAssetBalance::mock();

        assert!(details_state(WalletType::Multicoin, Chain::Ethereum, &plain, &balance, &[], true, 2).shows_price_alerts);
        assert!(!details_state(WalletType::Multicoin, Chain::Ethereum, &plain, &balance, &[], false, 2).shows_price_alerts);
        assert!(!details_state(WalletType::Multicoin, Chain::Ethereum, &plain, &balance, &[], true, 0).shows_price_alerts);
    }

    #[test]
    fn test_details_state_offers_earn_until_there_is_an_earn_balance() {
        let earn_enabled = metadata(true, false, false, true);

        assert!(details_state(WalletType::Multicoin, Chain::Ethereum, &earn_enabled, &GemAssetBalance::mock(), &[], true, 0).shows_earn);
        let earning = GemAssetBalance {
            earn: GemBigUint::from(100u32),
            ..GemAssetBalance::mock()
        };
        assert!(!details_state(WalletType::Multicoin, Chain::Ethereum, &earn_enabled, &earning, &[], true, 0).shows_earn);
        assert!(
            !details_state(
                WalletType::Multicoin,
                Chain::Ethereum,
                &metadata(true, false, false, false),
                &GemAssetBalance::mock(),
                &[],
                true,
                0
            )
            .shows_earn
        );
    }

    #[test]
    fn test_details_state_empty_transactions_prefer_buy_then_swap() {
        assert_eq!(
            state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, true, true, false), &[]).empty_transactions_action,
            Some(GemAssetEmptyAction::Buy)
        );
        assert_eq!(
            state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, false, true, false), &[]).empty_transactions_action,
            Some(GemAssetEmptyAction::Swap)
        );
        assert_eq!(
            state(WalletType::Multicoin, Chain::Ethereum, &metadata(true, false, false, false), &[]).empty_transactions_action,
            None
        );
    }
}
