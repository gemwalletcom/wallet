use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::{
    Asset, AssetBasic, AssetId, AssetMetaData, AssetPrice, AssetProperties, AssetScore, BalanceMetadata, BannerEvent, Chain, ChainAsset, ConfigVersions, Currency, PerpetualProvider, PriceAlert, StakeChain, VerificationStatus, Wallet,
    WalletType,
};

use super::model::{
    AssetList, GemAssetAction, GemAssetBalanceScope, GemAssetDetailRow, GemAssetDetailSection, GemAssetDetailsState, GemAssetEmptyAction, GemAssetFilter, GemAssetListRow, GemAssetListRowInput, GemAssetMenuAction, GemAssetMenuInput,
    GemAssetNetworkDestination, GemAssetRowStyle, GemAssetRowText, GemAssetSectionIds, GemAssetSubtitleStyle, GemAssetText, GemAssetTitleStyle, GemAssetTrailingStyle, GemHeaderActions, GemHeaderButton, GemHeaderButtonKind, GemPriceRow,
    GemSelectAssetFlow, GemSelectAssetScope, GemSelectAssetSection, GemSelectAssetTitle, GemSelectAssetType, GemSelectRowAction, GemWalletSearchCounts, GemWalletSearchLimits, GemWalletSearchPhase, GemWalletSearchState,
};
use crate::config::search_config::{ASSETS_INITIAL_LIMIT, ASSETS_SEARCH_LIMIT, NFTS_PREVIEW_LIMIT, PERPETUALS_PREVIEW_LIMIT, RESULTS_LIMIT};
use crate::config::stake::EARN_OFFERED;
use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigUint;
use crate::models::list::{GemListRow, GemListRowIcon, GemListRowTitle, GemListSectionTitle};
use crate::percentage::GemPercentageStyle;
use crate::perpetual::GemPerpetual;
use crate::precision::GemCurrencyStyle;
use crate::services::balance::rules::{balance_amount, balance_resource_rows};
use crate::services::balance::{GemAssetBalance, GemAssetBalanceRow, GemBalanceResource, GemBalanceRow, GemBalanceRowValue};
use crate::services::nft::rules::nft_chains;
use crate::services::price::rules::has_price;
use crate::services::price_alert::rules::{displayed_price_alert_ids, price_alert_toggle};
use number_formatter::CryptoFiatConverter;
use swapper::AssetList as SwapAssetList;

use crate::models::asset::{wallet_asset_is_enabled, wallet_default_assets};
use crate::services::collections::{missing, missing_by, unique, unique_by};
use primitives::AssetType;

pub fn menu_actions(input: &GemAssetMenuInput) -> Vec<GemAssetMenuAction> {
    [
        Some(GemAssetMenuAction::Pin { is_pinned: input.is_pinned }),
        input.offers_hide.then_some(GemAssetMenuAction::Hide),
        (input.offers_add_to_wallet && !input.is_balance_enabled).then_some(GemAssetMenuAction::AddToWallet),
        (!input.address.is_empty()).then(|| GemAssetMenuAction::CopyAddress { address: input.address.clone() }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn asset_list_versions(versions: &ConfigVersions) -> [(AssetList, i32); 3] {
    [(AssetList::Buy, versions.fiat_on_ramp_assets), (AssetList::Sell, versions.fiat_off_ramp_assets), (AssetList::Swap, versions.swap_assets)]
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
        .filter_map(|asset| asset.price.as_ref().map(|price| AssetPrice::new(asset.asset.id.clone(), price.price, price.price_change_percentage_24h, price.updated_at)))
        .collect()
}

pub fn default_asset(chain: Chain, asset_type: AssetType) -> Option<Asset> {
    wallet_default_assets(chain).into_iter().find(|asset| asset.asset_type == asset_type)
}

pub fn default_asset_basic(asset: Asset) -> AssetBasic {
    let asset_id = asset.id.clone();
    AssetBasic::new(asset, AssetProperties::default(asset_id.clone()), AssetScore::new(asset_id.default_rank()))
}

pub fn merge_assets(assets: Vec<AssetBasic>, tokens: Vec<AssetBasic>) -> Vec<AssetBasic> {
    unique_by(assets.into_iter().chain(tokens), |asset| asset.asset.id.clone())
}

pub fn default_assets() -> Vec<AssetBasic> {
    Chain::all()
        .into_iter()
        .flat_map(|chain| std::iter::once(Asset::from_chain(chain)).chain(wallet_default_assets(chain)))
        .map(default_asset_basic)
        .collect()
}

pub fn changed_assets(assets: Vec<AssetBasic>, stored: &[AssetBasic]) -> Vec<AssetBasic> {
    let stored: HashMap<&AssetId, &AssetBasic> = stored.iter().map(|basic| (&basic.asset.id, basic)).collect();
    assets
        .into_iter()
        .filter(|basic| match stored.get(&basic.asset.id) {
            Some(current) => !is_persisted_same(basic, current),
            None => true,
        })
        .collect()
}

fn is_persisted_same(left: &AssetBasic, right: &AssetBasic) -> bool {
    left.asset == right.asset && left.properties == right.properties && left.score == right.score
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
    .partition(|asset_id| wallet_asset_is_enabled(asset_id.clone(), wallet.wallet_type))
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

fn with_filter(mut flow: GemSelectAssetFlow, filter: Option<GemAssetFilter>) -> GemSelectAssetFlow {
    flow.filters.extend(filter);
    flow
}

pub fn asset_text(asset: &Asset) -> GemAssetText {
    let network_name = ChainAsset::from_chain(asset.chain()).network_name;
    GemAssetText {
        title: match asset.name == asset.symbol {
            true => asset.name.clone(),
            false => format!("{} ({})", asset.name, asset.symbol),
        },
        subtitle_symbol: (asset.name != asset.symbol).then(|| asset.symbol.clone()),
        network_full_name: match asset.id.is_native() {
            true => network_name.clone(),
            false => format!("{} ({})", network_name, asset.asset_type.as_ref()),
        },
        network_name,
    }
}

pub fn asset_row_text(asset: &Asset, style: GemAssetRowStyle) -> GemAssetRowText {
    let chain_asset = ChainAsset::from_chain(asset.chain());
    let title = match style.title {
        GemAssetTitleStyle::Asset => asset.name.clone(),
        GemAssetTitleStyle::CanonicalAsset => match asset.id.is_native() {
            true => chain_asset.asset.name.clone(),
            false => asset.name.clone(),
        },
        GemAssetTitleStyle::Network => chain_asset.network_name.clone(),
    };
    GemAssetRowText {
        symbol: (style.shows_symbol && title != asset.symbol).then(|| asset.symbol.clone()),
        network: (style.subtitle == GemAssetSubtitleStyle::Network && !asset.id.is_native()).then(|| chain_asset.network_name.clone()),
        title,
    }
}

pub fn asset_list_row(input: GemAssetListRowInput) -> GemAssetListRow {
    let GemAssetListRowInput {
        asset,
        balance,
        scope,
        price,
        change,
        currency,
        style,
    } = input;
    let value = match scope {
        GemAssetBalanceScope::Total => balance.total(),
        GemAssetBalanceScope::Available => balance.available,
    };
    GemAssetListRow {
        text: asset_row_text(&asset, style),
        price: price_row(price, change, currency.clone()),
        amount: crate::services::balance::rules::balance_amount_styled(&value, &asset, crate::precision::GemValueStyle::Short),
        fiat: fiat_amount(&asset, &value, price, currency),
        has_balance: value > GemBigUint::ZERO,
    }
}

pub fn wallet_asset_row_style() -> GemAssetRowStyle {
    GemAssetRowStyle {
        title: GemAssetTitleStyle::CanonicalAsset,
        shows_symbol: false,
        subtitle: GemAssetSubtitleStyle::Price,
        trailing: GemAssetTrailingStyle::Balance,
    }
}

fn select_asset_title(select_type: &GemSelectAssetType) -> GemSelectAssetTitle {
    match select_type {
        GemSelectAssetType::Send => GemSelectAssetTitle::Send,
        GemSelectAssetType::Receive => GemSelectAssetTitle::Receive,
        GemSelectAssetType::ReceiveCollection => GemSelectAssetTitle::ReceiveCollection,
        GemSelectAssetType::Buy => GemSelectAssetTitle::Buy,
        GemSelectAssetType::SwapPay => GemSelectAssetTitle::SwapPay,
        GemSelectAssetType::SwapReceive { .. } => GemSelectAssetTitle::SwapReceive,
        GemSelectAssetType::Payment { .. } => GemSelectAssetTitle::PayWith,
        GemSelectAssetType::Manage => GemSelectAssetTitle::ManageTokenList,
        GemSelectAssetType::PriceAlert => GemSelectAssetTitle::SelectAsset,
        GemSelectAssetType::Deposit => GemSelectAssetTitle::Deposit,
        GemSelectAssetType::Withdraw => GemSelectAssetTitle::Withdraw,
        GemSelectAssetType::WalletSearch | GemSelectAssetType::WalletSearchResults => GemSelectAssetTitle::Search,
    }
}

fn select_asset_section(select_type: &GemSelectAssetType) -> GemSelectAssetSection {
    match select_type {
        GemSelectAssetType::ReceiveCollection => GemSelectAssetSection::Networks,
        GemSelectAssetType::Send
        | GemSelectAssetType::Receive
        | GemSelectAssetType::Buy
        | GemSelectAssetType::SwapPay
        | GemSelectAssetType::SwapReceive { .. }
        | GemSelectAssetType::Payment { .. }
        | GemSelectAssetType::Manage
        | GemSelectAssetType::PriceAlert
        | GemSelectAssetType::Deposit
        | GemSelectAssetType::Withdraw
        | GemSelectAssetType::WalletSearch
        | GemSelectAssetType::WalletSearchResults => GemSelectAssetSection::Assets,
    }
}

pub fn select_asset_flow(select_type: GemSelectAssetType, swap_receive_assets: Option<SwapAssetList>) -> GemSelectAssetFlow {
    let style = |shows_symbol: bool, subtitle: GemAssetSubtitleStyle, trailing: GemAssetTrailingStyle| GemAssetRowStyle {
        title: GemAssetTitleStyle::CanonicalAsset,
        shows_symbol,
        subtitle,
        trailing,
    };
    let title = select_asset_title(&select_type);
    let assets_section = select_asset_section(&select_type);
    let flow = |row_action: GemSelectRowAction, action: Option<GemAssetAction>| GemSelectAssetFlow {
        title,
        assets_section,
        row_style: style(false, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Balance),
        row_action,
        action,
        scope: GemSelectAssetScope::Wallet,
        filters: action.map(|action| action.filters()).unwrap_or_default(),
        enables_price_alert: false,
        network_search: false,
        chain_filter: false,
        recents: false,
        popular_section: false,
        balance_filter: false,
        add_custom_token: false,
        display_asset: None,
    };
    match select_type {
        GemSelectAssetType::Send => GemSelectAssetFlow {
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Send))
        },
        GemSelectAssetType::Receive => GemSelectAssetFlow {
            row_style: style(true, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Copy),
            network_search: true,
            chain_filter: true,
            recents: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Receive))
        },
        GemSelectAssetType::ReceiveCollection => with_filter(
            GemSelectAssetFlow {
                row_style: GemAssetRowStyle {
                    title: GemAssetTitleStyle::Network,
                    ..style(false, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Copy)
                },
                network_search: true,
                recents: true,
                ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Receive))
            },
            Some(GemAssetFilter::asset_ids(nft_chains().into_iter().map(AssetId::from_chain).collect())),
        ),
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
        GemSelectAssetType::SwapReceive { .. } => with_filter(
            GemSelectAssetFlow {
                network_search: true,
                chain_filter: true,
                recents: true,
                ..flow(GemSelectRowAction::Select, Some(GemAssetAction::SwapReceive))
            },
            swap_receive_assets.map(GemAssetFilter::from),
        ),
        GemSelectAssetType::Payment { asset_ids } => with_filter(flow(GemSelectRowAction::Select, None), Some(GemAssetFilter::asset_ids(asset_ids))),
        GemSelectAssetType::Manage => with_filter(
            GemSelectAssetFlow {
                row_style: style(true, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Toggle),
                network_search: true,
                chain_filter: true,
                balance_filter: true,
                add_custom_token: true,
                ..flow(GemSelectRowAction::Toggle, None)
            },
            Some(GemAssetFilter::Enabled),
        ),
        GemSelectAssetType::PriceAlert => with_filter(
            GemSelectAssetFlow {
                row_style: style(true, GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::None),
                enables_price_alert: true,
                network_search: true,
                chain_filter: true,
                popular_section: true,
                scope: GemSelectAssetScope::AllAssets,
                ..flow(GemSelectRowAction::Select, None)
            },
            Some(GemAssetFilter::Enabled),
        ),
        GemSelectAssetType::Deposit => with_filter(
            flow(GemSelectRowAction::Navigate, None),
            Some(GemAssetFilter::asset_ids(vec![GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset().id])),
        ),
        GemSelectAssetType::Withdraw => with_filter(
            GemSelectAssetFlow {
                display_asset: Some(GemPerpetual::new(PerpetualProvider::Hypercore).deposit_asset()),
                ..flow(GemSelectRowAction::Navigate, None)
            },
            Some(GemAssetFilter::asset_ids(vec![HYPERCORE_PERPETUAL_USDC.id.clone()])),
        ),
        GemSelectAssetType::WalletSearch => GemSelectAssetFlow {
            row_style: GemAssetRowStyle {
                title: GemAssetTitleStyle::Asset,
                ..style(false, GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::Balance)
            },
            network_search: true,
            recents: true,
            add_custom_token: true,
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Open))
        },
        GemSelectAssetType::WalletSearchResults => GemSelectAssetFlow {
            row_style: GemAssetRowStyle {
                title: GemAssetTitleStyle::Asset,
                ..style(false, GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::Balance)
            },
            ..flow(GemSelectRowAction::Navigate, Some(GemAssetAction::Open))
        },
    }
}

pub fn asset_sections(ids: Vec<AssetId>, pinned_ids: Vec<AssetId>, shows_popular: bool, popular_ids: Vec<AssetId>) -> GemAssetSectionIds {
    let pinned_ids: HashSet<AssetId> = pinned_ids.into_iter().collect();
    let popular_ids: HashSet<AssetId> = match shows_popular {
        true => popular_ids.into_iter().collect(),
        false => HashSet::new(),
    };
    ids.into_iter().fold(GemAssetSectionIds::default(), |mut sections, id| {
        match (pinned_ids.contains(&id), popular_ids.contains(&id)) {
            (true, _) => sections.pinned.push(id),
            (false, true) => sections.popular.push(id),
            (false, false) => sections.assets.push(id),
        }
        sections
    })
}

pub fn applied_filters(flow: &GemSelectAssetFlow, chains: Vec<Chain>, has_balance: bool) -> Vec<GemAssetFilter> {
    let chains = (!chains.is_empty()).then_some(GemAssetFilter::Chains { chains });
    let balance = (has_balance && flow.balance_filter).then_some(GemAssetFilter::HasBalance);
    let mut filters = flow.filters.clone();
    for filter in chains.into_iter().chain(balance) {
        if !filters.contains(&filter) {
            filters.push(filter);
        }
    }
    filters
}

pub fn wallet_search_state(counts: &GemWalletSearchCounts, is_loading: bool) -> GemWalletSearchState {
    let shown = counts.recents + counts.pinned_assets + counts.assets + counts.pinned_perpetuals + counts.perpetuals + counts.lists + counts.nfts;
    GemWalletSearchState {
        phase: match (shown > 0, is_loading) {
            (true, _) => GemWalletSearchPhase::Results,
            (false, true) => GemWalletSearchPhase::Loading,
            (false, false) => GemWalletSearchPhase::Empty,
        },
        shows_recents: counts.recents > 0,
        shows_pinned: counts.pinned_assets > 0 || counts.pinned_perpetuals > 0,
        shows_assets: counts.assets > 0,
        shows_pinned_perpetuals: counts.pinned_perpetuals > 0,
        shows_perpetuals: counts.perpetuals > 0,
        shows_lists: counts.lists > 0,
        shows_nfts: counts.nfts > 0,
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

pub fn asset_title(asset: &Asset) -> String {
    match asset.id.is_native() {
        true => Asset::from_chain(asset.chain()).name,
        false => asset.name.clone(),
    }
}

pub fn fiat_value(asset: &Asset, balance: &GemAssetBalance, price: Option<f64>, currency: Currency) -> Option<GemFormattedNumber> {
    fiat_amount(asset, &balance.total(), price, currency)
}

pub fn fiat_amount_of(asset: &Asset, value: &num_bigint::BigUint, price: Option<f64>, currency: Currency) -> Option<GemFormattedNumber> {
    fiat_amount(asset, &GemBigUint::from(value.clone()), price, currency)
}

fn fiat_amount(asset: &Asset, value: &GemBigUint, price: Option<f64>, currency: Currency) -> Option<GemFormattedNumber> {
    let value: f64 = CryptoFiatConverter::to_fiat(&value.to_string(), asset.decimals as u32, price?).ok()?.parse().ok()?;
    (value > 0.0).then(|| GemFormattedNumber::currency(value, currency, GemCurrencyStyle::Currency))
}

pub fn price_row(price: Option<f64>, change: Option<f64>, currency: Currency) -> GemPriceRow {
    GemPriceRow {
        price: price.filter(|price| *price > 0.0).map(|price| GemFormattedNumber::currency(price, currency, GemCurrencyStyle::Currency)),
        change: change.map(|change| GemFormattedNumber::percentage(change, GemPercentageStyle::Signed).toned()),
    }
}

fn apr(apr: Option<f64>) -> Option<GemFormattedNumber> {
    apr.filter(|apr| *apr > 0.0).map(|apr| GemFormattedNumber::percentage(apr, GemPercentageStyle::Unsigned))
}

pub fn balance_rows(asset: &Asset, metadata: &AssetMetaData, balance: &GemAssetBalance) -> Vec<GemAssetBalanceRow> {
    balance
        .detail_rows(asset.chain(), metadata.is_stake_enabled)
        .into_iter()
        .map(|row| {
            let value = match &row {
                GemBalanceRow::Staked { value } if *value == GemBigUint::ZERO => GemBalanceRowValue::Apr { apr: apr(metadata.staking_apr) },
                GemBalanceRow::Earn { value } if *value == GemBigUint::ZERO => GemBalanceRowValue::Apr { apr: apr(metadata.earn_apr) },
                GemBalanceRow::Available { .. } | GemBalanceRow::Staked { .. } | GemBalanceRow::Earn { .. } | GemBalanceRow::PendingUnconfirmed { .. } | GemBalanceRow::Reserved { .. } => {
                    GemBalanceRowValue::Amount { amount: balance_amount(&row.value(), asset) }
                }
            };
            GemAssetBalanceRow { row, value }
        })
        .collect()
}

pub struct DetailsSectionsInput<'a> {
    pub wallet_type: WalletType,
    pub asset: &'a Asset,
    pub metadata: &'a AssetMetaData,
    pub balance: &'a GemAssetBalance,
    pub price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub currency: Currency,
    pub price_alerts: &'a [PriceAlert],
    pub fee_balance_metadata: Option<BalanceMetadata>,
}

pub fn details_sections(input: DetailsSectionsInput) -> Vec<GemAssetDetailSection> {
    let DetailsSectionsInput {
        wallet_type,
        asset,
        metadata,
        balance,
        price,
        price_change_percentage_24h,
        currency,
        price_alerts,
        fee_balance_metadata,
    } = input;
    let chain = asset.chain();
    let displayed_alerts = displayed_price_alert_ids(price_alerts.to_vec()).len();
    let quoted = price_row(price, price_change_percentage_24h, currency.clone());
    let row = |row: GemListRow| GemAssetDetailRow::Row { row };
    let link = |title: GemListRowTitle, value: Option<String>, icon: GemListRowIcon| row(GemListRow::Link { title, value, icon });
    let section = |title: GemListSectionTitle, rows: Vec<GemAssetDetailRow>| (!rows.is_empty()).then_some(GemAssetDetailSection { title, rows });
    let pin = match metadata.is_pinned {
        true => link(GemListRowTitle::Unpin, None, GemListRowIcon::Unpin),
        false => link(GemListRowTitle::Pin, None, GemListRowIcon::Pin),
    };
    let shows_earn = EARN_OFFERED && metadata.is_earn_enabled && wallet_type != WalletType::View && balance.earn == GemBigUint::ZERO;
    let shows_resources = StakeChain::from_str(chain.as_ref()).is_ok_and(|stake_chain| stake_chain.get_uses_freeze());
    [
        section(
            GemListSectionTitle::Manage,
            match metadata.is_balance_enabled {
                true => vec![],
                false => vec![pin, link(GemListRowTitle::AddToWallet, None, GemListRowIcon::AddToWallet)],
            },
        ),
        section(
            GemListSectionTitle::None,
            [
                Some(row(GemListRow::Quote {
                    title: GemListRowTitle::Price,
                    value: quoted.price,
                    change: quoted.change,
                })),
                (has_price(price) && displayed_alerts > 0).then(|| link(GemListRowTitle::PriceAlerts, Some(displayed_alerts.to_string()), GemListRowIcon::None)),
                Some(row(GemListRow::Network {
                    title: GemListRowTitle::Network,
                    chain,
                    name: asset_text(asset).network_full_name,
                })),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        section(GemListSectionTitle::Balances, balance_rows(asset, metadata, balance).into_iter().map(|row| GemAssetDetailRow::Balance { row }).collect()),
        section(
            GemListSectionTitle::None,
            match shows_earn {
                true => vec![row(crate::services::stake::rules::earn_apr_row(&[], metadata.earn_apr))],
                false => vec![],
            },
        ),
        section(
            GemListSectionTitle::Resources,
            match shows_resources {
                true => balance_resource_rows(fee_balance_metadata)
                    .into_iter()
                    .map(|resource| {
                        row(GemListRow::Text {
                            title: match resource.resource {
                                GemBalanceResource::Energy => GemListRowTitle::Energy,
                                GemBalanceResource::Bandwidth => GemListRowTitle::Bandwidth,
                            },
                            value: resource.text,
                        })
                    })
                    .collect(),
                false => vec![],
            },
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

pub fn details_state(wallet_type: WalletType, metadata: &AssetMetaData, banner_events: &[BannerEvent], price_alerts: &[PriceAlert]) -> GemAssetDetailsState {
    let is_view_only = wallet_type == WalletType::View;
    let buttons_enabled = !banner_events.iter().any(|event| matches!(event, BannerEvent::ActivateAsset | BannerEvent::AccountBlockedMultiSignature));
    let button = |kind: GemHeaderButtonKind, shows: bool| shows.then_some(GemHeaderButton { kind, is_enabled: buttons_enabled });
    GemAssetDetailsState {
        is_view_only,
        header_actions: if is_view_only {
            GemHeaderActions::WatchOnly
        } else {
            GemHeaderActions::Buttons {
                buttons: [
                    button(GemHeaderButtonKind::Send, true),
                    button(GemHeaderButtonKind::Receive, true),
                    button(GemHeaderButtonKind::Buy, metadata.is_buy_enabled),
                    button(GemHeaderButtonKind::Swap, metadata.is_swap_enabled),
                ]
                .into_iter()
                .flatten()
                .collect(),
            }
        },
        shows_banners: !banner_events.is_empty(),
        price_alert: price_alert_toggle(price_alerts),
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

    #[test]
    fn test_changed_assets_drops_identical_rows_and_keeps_real_edits() {
        let ethereum = default_asset_basic(Asset::from_chain(Chain::Ethereum));
        let bitcoin = default_asset_basic(Asset::from_chain(Chain::Bitcoin));
        let stored = vec![ethereum.clone()];

        let mut renamed = ethereum.clone();
        renamed.asset.name = "Ethereum Mainnet".to_string();
        let mut reranked = ethereum.clone();
        reranked.score.rank += 5;
        let mut unstakeable = ethereum.clone();
        unstakeable.properties.is_stakeable = !ethereum.properties.is_stakeable;

        assert!(changed_assets(vec![ethereum.clone()], &stored).is_empty(), "an unchanged row is not written again");
        assert_eq!(changed_assets(vec![bitcoin.clone()], &stored), vec![bitcoin]);
        assert_eq!(changed_assets(vec![renamed.clone()], &stored), vec![renamed]);
        assert_eq!(changed_assets(vec![reranked.clone()], &stored), vec![reranked], "rank is persisted, so it counts as a change");
        assert_eq!(changed_assets(vec![unstakeable.clone()], &stored), vec![unstakeable], "properties are persisted too");
        let mut imaged = ethereum.clone();
        imaged.properties.has_image = !ethereum.properties.has_image;
        assert_eq!(changed_assets(vec![imaged.clone()], &stored), vec![imaged], "every property both stores keep counts as a change");
        assert!(changed_assets(vec![], &stored).is_empty());
    }

    #[test]
    fn test_asset_text_names_the_asset_and_its_network_once() {
        let ethereum = asset_text(&Asset::from_chain(Chain::Ethereum));
        assert_eq!(ethereum.title, "Ethereum (ETH)");
        assert_eq!(ethereum.subtitle_symbol.as_deref(), Some("ETH"));
        assert_eq!(ethereum.network_full_name, "Ethereum", "a coin's network needs no type");

        let token = Asset::new(AssetId::from_token(Chain::Ethereum, "0xusdc"), "USDC".into(), "USDC".into(), 6, primitives::AssetType::ERC20);
        let usdc = asset_text(&token);
        assert_eq!(usdc.title, "USDC", "a name that already is the symbol is not repeated");
        assert_eq!(usdc.subtitle_symbol, None);
        assert_eq!(usdc.network_full_name, "Ethereum (ERC20)");
    }

    #[test]
    fn test_a_list_row_carries_the_balance_its_fiat_and_whether_there_is_any() {
        let usdc = Asset::mock_ethereum_usdc();
        let row = |balance: GemAssetBalance, scope, price| {
            asset_list_row(GemAssetListRowInput {
                asset: usdc.clone(),
                balance,
                scope,
                price,
                change: Some(-2.5),
                currency: Currency::USD,
                style: wallet_asset_row_style(),
            })
        };
        let held = GemAssetBalance {
            staked: GemBigUint::from(2_000_000u32),
            ..GemAssetBalance::mock_with_available(1_000_000)
        };

        let total = row(held.clone(), GemAssetBalanceScope::Total, Some(1.0));
        assert_eq!(total.amount.value, 3.0);
        assert_eq!(total.amount.unit, crate::formatted_number::GemNumberUnit::Symbol { symbol: usdc.symbol.clone() });
        assert_eq!(total.fiat.expect("a priced balance is worth something").value, 3.0);
        assert_eq!(total.price, price_row(Some(1.0), Some(-2.5), Currency::USD), "the row's price is the one the detail screen shows");
        assert!(total.has_balance);

        let available = row(held.clone(), GemAssetBalanceScope::Available, Some(1.0));
        assert_eq!(available.amount.value, 1.0, "the buy screen spends what is available, not what is staked");

        assert_eq!(row(held, GemAssetBalanceScope::Total, None).fiat, None, "an unpriced asset is worth nothing the row can name");

        let empty = row(GemAssetBalance::mock(), GemAssetBalanceScope::Total, Some(1.0));
        assert!(!empty.has_balance, "an empty balance greys the row on both apps");
        assert_eq!(empty.fiat, None);
    }

    #[test]
    fn test_a_row_titled_by_its_network_reads_the_network_not_the_coin() {
        let style = |title| GemAssetRowStyle {
            title,
            shows_symbol: true,
            subtitle: GemAssetSubtitleStyle::Network,
            trailing: GemAssetTrailingStyle::Balance,
        };
        let ton = Asset::from_chain(Chain::Ton);

        assert_eq!(asset_row_text(&ton, style(GemAssetTitleStyle::Network)).title, "TON");
        assert_eq!(asset_row_text(&ton, style(GemAssetTitleStyle::Asset)).title, "Gram");
        assert_eq!(asset_row_text(&ton, style(GemAssetTitleStyle::CanonicalAsset)).title, "Gram");
    }

    #[test]
    fn test_a_row_shows_the_symbol_only_when_it_adds_to_the_title_it_shows() {
        let style = |shows_symbol, title| GemAssetRowStyle {
            title,
            shows_symbol,
            subtitle: GemAssetSubtitleStyle::Network,
            trailing: GemAssetTrailingStyle::Balance,
        };
        let usdc = Asset::new(AssetId::from_token(Chain::Ethereum, "0xusdc"), "USDC".into(), "USDC".into(), 6, primitives::AssetType::ERC20);
        let ethereum = Asset::from_chain(Chain::Ethereum);

        assert_eq!(asset_row_text(&usdc, style(true, GemAssetTitleStyle::Asset)).symbol, None, "a name that already is the symbol is not repeated");
        assert_eq!(asset_row_text(&ethereum, style(true, GemAssetTitleStyle::Asset)).symbol.as_deref(), Some("ETH"));
        assert_eq!(asset_row_text(&ethereum, style(false, GemAssetTitleStyle::Asset)).symbol, None);
        assert_eq!(
            asset_row_text(&usdc, style(true, GemAssetTitleStyle::Network)).symbol.as_deref(),
            Some("USDC"),
            "the title the row shows is what the symbol would repeat"
        );
    }

    #[test]
    fn test_a_row_names_its_network_underneath_only_for_a_token() {
        let style = |subtitle| GemAssetRowStyle {
            title: GemAssetTitleStyle::Asset,
            shows_symbol: false,
            subtitle,
            trailing: GemAssetTrailingStyle::Balance,
        };
        let usdc = Asset::new(AssetId::from_token(Chain::Ethereum, "0xusdc"), "USDC".into(), "USDC".into(), 6, primitives::AssetType::ERC20);

        assert_eq!(asset_row_text(&usdc, style(GemAssetSubtitleStyle::Network)).network.as_deref(), Some("Ethereum"));
        assert_eq!(asset_row_text(&usdc, style(GemAssetSubtitleStyle::Price)).network, None);
        assert_eq!(
            asset_row_text(&Asset::from_chain(Chain::Ethereum), style(GemAssetSubtitleStyle::Network)).network,
            None,
            "a coin's row already names its network"
        );
    }

    #[test]
    fn test_asset_menu_offers_pin_always_and_the_rest_only_when_they_apply() {
        let input = GemAssetMenuInput {
            is_pinned: false,
            is_balance_enabled: false,
            address: "0xabc".to_string(),
            offers_hide: true,
            offers_add_to_wallet: true,
        };

        assert_eq!(
            menu_actions(&input),
            vec![
                GemAssetMenuAction::Pin { is_pinned: false },
                GemAssetMenuAction::Hide,
                GemAssetMenuAction::AddToWallet,
                GemAssetMenuAction::CopyAddress { address: "0xabc".to_string() },
            ]
        );
        assert_eq!(
            menu_actions(&GemAssetMenuInput { is_balance_enabled: true, ..input.clone() }),
            vec![GemAssetMenuAction::Pin { is_pinned: false }, GemAssetMenuAction::Hide, GemAssetMenuAction::CopyAddress { address: "0xabc".to_string() },],
            "an asset already in the wallet cannot be added again"
        );
        assert_eq!(
            menu_actions(&GemAssetMenuInput {
                address: String::new(),
                offers_hide: false,
                offers_add_to_wallet: false,
                ..input
            }),
            vec![GemAssetMenuAction::Pin { is_pinned: false }],
            "there is nothing to copy without an address"
        );
    }
    use super::*;
    use crate::services::price_alert::rules::GemPriceAlertToggle;

    #[test]
    fn test_each_select_flow_decides_its_row_action_and_recent_activity() {
        let row = |select_type: GemSelectAssetType| (select_type.flow().row_action, select_type.flow().action);
        assert_eq!(row(GemSelectAssetType::Send), (GemSelectRowAction::Navigate, Some(GemAssetAction::Send)));
        assert_eq!(row(GemSelectAssetType::Receive), (GemSelectRowAction::Navigate, Some(GemAssetAction::Receive)));
        assert_eq!(row(GemSelectAssetType::ReceiveCollection), (GemSelectRowAction::Navigate, Some(GemAssetAction::Receive)));
        assert_eq!(row(GemSelectAssetType::Buy), (GemSelectRowAction::Navigate, Some(GemAssetAction::Buy)));
        assert_eq!(row(GemSelectAssetType::SwapPay), (GemSelectRowAction::Select, Some(GemAssetAction::SwapPay)));
        assert_eq!(row(GemSelectAssetType::SwapReceive { pay_asset_id: None }), (GemSelectRowAction::Select, Some(GemAssetAction::SwapReceive)));
        assert_eq!(row(GemSelectAssetType::Payment { asset_ids: vec![] }), (GemSelectRowAction::Select, None));
        assert_eq!(row(GemSelectAssetType::Manage), (GemSelectRowAction::Toggle, None));
        assert_eq!(row(GemSelectAssetType::PriceAlert), (GemSelectRowAction::Select, None));
        assert_eq!(row(GemSelectAssetType::Deposit), (GemSelectRowAction::Navigate, None));
        assert_eq!(row(GemSelectAssetType::Withdraw), (GemSelectRowAction::Navigate, None));
        assert_eq!(row(GemSelectAssetType::WalletSearch), (GemSelectRowAction::Navigate, Some(GemAssetAction::Open)));
        assert_eq!(row(GemSelectAssetType::WalletSearchResults), (GemSelectRowAction::Navigate, Some(GemAssetAction::Open)));
    }

    #[test]
    fn test_the_wallet_list_row_names_a_native_asset_by_its_chain_and_hides_the_symbol() {
        let row = wallet_asset_row_style();
        assert_eq!(row.title, GemAssetTitleStyle::CanonicalAsset);
        assert!(!row.shows_symbol, "the wallet list repeats no symbol beside the name");
        assert_eq!((row.subtitle, row.trailing), (GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::Balance));
    }

    #[test]
    fn test_each_select_flow_shows_the_row_both_apps_draw() {
        let row = |select_type: GemSelectAssetType| {
            let row = select_type.flow().row_style;
            (row.title, row.shows_symbol, row.subtitle, row.trailing)
        };
        let balance = (GemAssetTitleStyle::CanonicalAsset, false, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Balance);
        for select_type in [
            GemSelectAssetType::Send,
            GemSelectAssetType::Buy,
            GemSelectAssetType::SwapPay,
            GemSelectAssetType::SwapReceive { pay_asset_id: None },
            GemSelectAssetType::Deposit,
            GemSelectAssetType::Withdraw,
        ] {
            assert_eq!(row(select_type), balance);
        }
        assert_eq!(row(GemSelectAssetType::Receive), (GemAssetTitleStyle::CanonicalAsset, true, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Copy));
        assert_eq!(row(GemSelectAssetType::ReceiveCollection), (GemAssetTitleStyle::Network, false, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Copy));
        assert_eq!(row(GemSelectAssetType::Manage), (GemAssetTitleStyle::CanonicalAsset, true, GemAssetSubtitleStyle::Network, GemAssetTrailingStyle::Toggle));
        assert_eq!(row(GemSelectAssetType::PriceAlert), (GemAssetTitleStyle::CanonicalAsset, true, GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::None));
        let search = (GemAssetTitleStyle::Asset, false, GemAssetSubtitleStyle::Price, GemAssetTrailingStyle::Balance);
        assert_eq!(row(GemSelectAssetType::WalletSearch), search);
        assert_eq!(row(GemSelectAssetType::WalletSearchResults), search);
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
                ("display_asset", flow.display_asset.is_some()),
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
        assert_eq!(enabled(GemSelectAssetType::SwapReceive { pay_asset_id: None }), ["network_search", "chain_filter", "recents"]);
        assert!(enabled(GemSelectAssetType::Payment { asset_ids: vec![] }).is_empty());
        assert_eq!(enabled(GemSelectAssetType::Manage), ["network_search", "chain_filter", "balance_filter", "add_custom_token"]);
        assert_eq!(enabled(GemSelectAssetType::PriceAlert), ["network_search", "chain_filter", "popular_section", "enables_price_alert"]);
        assert!(enabled(GemSelectAssetType::Deposit).is_empty());
        assert_eq!(enabled(GemSelectAssetType::Withdraw), ["display_asset"]);
        assert_eq!(enabled(GemSelectAssetType::WalletSearch), ["network_search", "recents", "add_custom_token"]);
        assert!(enabled(GemSelectAssetType::WalletSearchResults).is_empty());
    }

    #[test]
    fn test_each_select_flow_lists_the_rows_its_action_can_offer() {
        let rows = |select_type: GemSelectAssetType| (select_type.flow().scope, select_type.flow().filters);
        assert_eq!(rows(GemSelectAssetType::Send), (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance]));
        assert_eq!(rows(GemSelectAssetType::Receive), (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled]));
        let (_, collection_filters) = rows(GemSelectAssetType::ReceiveCollection);
        assert_eq!(collection_filters[0], GemAssetFilter::Enabled);
        let GemAssetFilter::ChainsOrAssetIds {
            chains: collection_chains,
            asset_ids: collection_assets,
        } = &collection_filters[1]
        else {
            panic!("receive collection lists the NFT chains' native assets");
        };
        assert!(collection_chains.is_empty());
        assert!(collection_assets.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(!collection_assets.contains(&AssetId::from_chain(Chain::Bitcoin)));
        assert!(collection_assets.iter().all(AssetId::is_native));
        assert_eq!(rows(GemSelectAssetType::Buy), (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable]));
        assert_eq!(
            rows(GemSelectAssetType::SwapPay),
            (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance])
        );
        assert_eq!(
            rows(GemSelectAssetType::SwapReceive { pay_asset_id: None }),
            (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable])
        );
        assert_eq!(rows(GemSelectAssetType::Manage), (GemSelectAssetScope::Wallet, vec![GemAssetFilter::Enabled]));
        assert_eq!(rows(GemSelectAssetType::PriceAlert), (GemSelectAssetScope::AllAssets, vec![GemAssetFilter::Enabled]));
        assert_eq!(
            rows(GemSelectAssetType::Deposit),
            (
                GemSelectAssetScope::Wallet,
                vec![GemAssetFilter::ChainsOrAssetIds {
                    chains: Vec::new(),
                    asset_ids: vec![AssetId::from_token(Chain::Arbitrum, "0xaf88d065e77c8cC2239327C5EDb3A432268e5831")]
                }]
            )
        );
        assert_eq!(
            rows(GemSelectAssetType::Withdraw),
            (
                GemSelectAssetScope::Wallet,
                vec![GemAssetFilter::ChainsOrAssetIds {
                    chains: Vec::new(),
                    asset_ids: vec![HYPERCORE_PERPETUAL_USDC.id.clone()]
                }]
            )
        );
        assert_eq!(rows(GemSelectAssetType::WalletSearch), (GemSelectAssetScope::Wallet, Vec::new()));
        assert_eq!(rows(GemSelectAssetType::WalletSearchResults), (GemSelectAssetScope::Wallet, Vec::new()));
    }

    #[test]
    fn test_swap_receive_lists_the_pay_assets_swap_universe() {
        let pay_asset_id = AssetId::from_chain(Chain::Ethereum);
        let universe = SwapAssetList {
            chains: vec![Chain::Solana],
            asset_ids: vec![AssetId::from_token(Chain::SmartChain, "0x123")],
        };

        let flow = select_asset_flow(GemSelectAssetType::SwapReceive { pay_asset_id: Some(pay_asset_id) }, Some(universe.clone()));

        assert_eq!(
            flow.filters,
            vec![
                GemAssetFilter::Enabled,
                GemAssetFilter::Swappable,
                GemAssetFilter::ChainsOrAssetIds {
                    chains: universe.chains,
                    asset_ids: universe.asset_ids,
                },
            ]
        );
    }

    #[test]
    fn test_the_network_screen_lists_the_chain_tokens_without_its_coin() {
        use super::super::model::shows_on_network_assets;

        assert!(!shows_on_network_assets(AssetId::from_chain(Chain::Ethereum)));
        assert!(shows_on_network_assets(Asset::mock_ethereum_usdc().id));
    }

    #[test]
    fn test_network_assets_are_empty_only_when_every_section_is() {
        use super::super::model::GemNetworkAssetCounts;
        let counts = |pinned, unpinned, hidden| GemNetworkAssetCounts { pinned, unpinned, hidden }.sections();

        assert!(counts(0, 0, 0).shows_empty);
        assert!(!counts(0, 0, 1).shows_empty);
        assert!(counts(0, 0, 1).shows_hidden);
        assert!(counts(2, 0, 0).shows_pinned);
        assert!(!counts(2, 0, 0).shows_unpinned);
    }

    #[test]
    fn test_a_popular_asset_leaves_the_plain_bucket_and_a_pinned_one_wins_over_both() {
        let ids = vec![Chain::Bitcoin.as_asset_id(), Chain::Ethereum.as_asset_id(), Chain::Solana.as_asset_id()];
        let popular = vec![Chain::Ethereum.as_asset_id()];

        let shown = asset_sections(ids.clone(), vec![Chain::Bitcoin.as_asset_id()], true, popular.clone());
        assert_eq!(shown.pinned, vec![Chain::Bitcoin.as_asset_id()]);
        assert_eq!(shown.popular, vec![Chain::Ethereum.as_asset_id()]);
        assert_eq!(shown.assets, vec![Chain::Solana.as_asset_id()]);

        let hidden = asset_sections(ids, vec![], false, popular);
        assert!(hidden.popular.is_empty());
        assert_eq!(hidden.assets.len(), 3);
    }

    #[test]
    fn test_the_search_screen_shows_results_whenever_any_section_has_something() {
        let empty = GemWalletSearchCounts {
            recents: 0,
            pinned_assets: 0,
            assets: 0,
            pinned_perpetuals: 0,
            perpetuals: 0,
            lists: 0,
            nfts: 0,
        };

        assert_eq!(wallet_search_state(&empty, false).phase, GemWalletSearchPhase::Empty);
        assert_eq!(wallet_search_state(&empty, true).phase, GemWalletSearchPhase::Loading);
        assert_eq!(
            wallet_search_state(&GemWalletSearchCounts { nfts: 1, ..empty }, true).phase,
            GemWalletSearchPhase::Results,
            "a section with results is not a loading screen"
        );
        assert_eq!(wallet_search_state(&GemWalletSearchCounts { recents: 2, ..empty }, false).phase, GemWalletSearchPhase::Results);
        for counts in [
            GemWalletSearchCounts { lists: 1, ..empty },
            GemWalletSearchCounts { assets: 1, ..empty },
            GemWalletSearchCounts { pinned_assets: 1, ..empty },
            GemWalletSearchCounts { pinned_perpetuals: 1, ..empty },
            GemWalletSearchCounts { perpetuals: 1, ..empty },
        ] {
            assert_eq!(wallet_search_state(&counts, false).phase, GemWalletSearchPhase::Results, "every section keeps the empty state away");
        }
    }

    #[test]
    fn test_a_section_shows_exactly_when_it_counted_something() {
        let counts = GemWalletSearchCounts {
            recents: 0,
            pinned_assets: 0,
            assets: 3,
            pinned_perpetuals: 2,
            perpetuals: 0,
            lists: 1,
            nfts: 0,
        };
        let state = wallet_search_state(&counts, false);

        assert!(state.shows_assets && state.shows_lists && state.shows_pinned_perpetuals);
        assert!(state.shows_pinned, "a pinned perpetual alone still opens the pinned section");
        assert!(!state.shows_recents && !state.shows_perpetuals && !state.shows_nfts);
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
    fn test_network_destination_opens_the_native_asset_or_the_chain_list() {
        let ethereum = Chain::Ethereum;
        assert_eq!(network_destination(&Asset::mock_ethereum_usdc().id), Some(GemAssetNetworkDestination::Asset { asset: Asset::from_chain(ethereum) }));
        assert_eq!(network_destination(&AssetId::from_token(Chain::Tempo, "0x1")), Some(GemAssetNetworkDestination::Assets { chain: Chain::Tempo }));
        assert_eq!(network_destination(&AssetId::from_chain(ethereum)), Some(GemAssetNetworkDestination::Assets { chain: ethereum }));
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
        let multicoin = Wallet::mock_with_chains(&[Chain::Bitcoin, Chain::Doge, Chain::Near, Chain::Xrp, Chain::Ethereum, Chain::Near]);
        assert_eq!(token_chains(&multicoin), vec![Chain::Ethereum, Chain::Xrp, Chain::Near]);
        assert!(token_chains(&Wallet::mock_with_type(WalletType::Single, &[Chain::Bitcoin])).is_empty());
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
        let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Tempo]);
        assert!(can_open(&wallet, &AssetId::from_chain(Chain::Ethereum)));
        assert!(can_open(&wallet, &AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()))));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Bitcoin)));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Tempo)));
        assert!(can_open(&wallet, &AssetId::from(Chain::Tempo, Some("0x20c000000000000000000000c48d6a3bd5b7b0c2".to_string()))));
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
    use chrono::Utc;
    use primitives::{Chain, PriceAlertDirection, WalletType, currency::Currency};

    #[test]
    fn test_default_asset_basic_uses_default_rank_and_properties() {
        let native = default_asset_basic(Asset::from_chain(Chain::Ethereum));
        let token = default_asset_basic(Asset::mock_erc20());

        assert_eq!(native.score.rank, Chain::Ethereum.rank());
        assert!(native.score.rank > token.score.rank);
        assert!(native.properties.is_enabled);
        assert!(native.price.is_none());
    }

    #[test]
    fn test_merge_assets_keeps_the_backend_copy_of_a_token() {
        let merged = merge_assets(
            vec![
                default_asset_basic(Asset::from_chain(Chain::Ethereum)),
                AssetBasic {
                    score: AssetScore::new(34),
                    ..default_asset_basic(Asset::mock_erc20())
                },
            ],
            vec![
                AssetBasic {
                    score: AssetScore::new(15),
                    ..default_asset_basic(Asset::mock_erc20())
                },
                default_asset_basic(Asset::from_chain(Chain::Solana)),
            ],
        );

        assert_eq!(merged.iter().map(|asset| asset.score.rank).collect::<Vec<_>>(), vec![Chain::Ethereum.rank(), 34, Chain::Solana.rank()]);
    }

    #[test]
    fn test_default_balances_by_wallet_type() {
        let (enabled, disabled) = default_balances(&Wallet::mock_with_chains(&[Chain::Cosmos, Chain::Ethereum, Chain::Tron]));
        assert!(disabled.contains(&AssetId::from_chain(Chain::Cosmos)));
        assert!(enabled.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(wallet_default_assets(Chain::Tron).iter().all(|asset| enabled.contains(&asset.id)));

        let (enabled, disabled) = default_balances(&Wallet::mock_with_type(WalletType::Single, &[Chain::Cosmos]));
        assert_eq!(enabled, vec![AssetId::from_chain(Chain::Cosmos)]);
        assert!(disabled.is_empty());

        let (enabled, _) = default_balances(&Wallet::mock_with_type(WalletType::Single, &[Chain::Tempo]));
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

    fn state(wallet_type: WalletType, metadata: &AssetMetaData, banner_events: &[BannerEvent]) -> GemAssetDetailsState {
        details_state(wallet_type, metadata, banner_events, &[])
    }

    fn sections(asset: &Asset, metadata: &AssetMetaData, balance: &GemAssetBalance, price: Option<f64>, price_alerts: &[PriceAlert]) -> Vec<GemAssetDetailSection> {
        details_sections(DetailsSectionsInput {
            wallet_type: WalletType::Multicoin,
            asset,
            metadata,
            balance,
            price,
            price_change_percentage_24h: None,
            currency: Currency::USD,
            price_alerts,
            fee_balance_metadata: None,
        })
    }

    fn section(sections: &[GemAssetDetailSection], title: GemListSectionTitle) -> Vec<Vec<GemAssetDetailRow>> {
        sections.iter().filter(|section| section.title == title).map(|section| section.rows.clone()).collect()
    }

    fn buttons(state: &GemAssetDetailsState) -> Vec<GemHeaderButton> {
        match &state.header_actions {
            GemHeaderActions::Buttons { buttons } => buttons.clone(),
            GemHeaderActions::WatchOnly => panic!("the header offers no buttons"),
        }
    }

    fn kinds(state: &GemAssetDetailsState) -> Vec<GemHeaderButtonKind> {
        buttons(state).into_iter().map(|button| button.kind).collect()
    }

    #[test]
    fn test_an_empty_stake_balance_shows_the_rate_it_would_earn() {
        use crate::precision::GemValueStyle;
        use num_bigint::BigUint;
        let asset = Asset::from_chain(Chain::Cosmos);
        let metadata = AssetMetaData {
            is_stake_enabled: true,
            staking_apr: Some(15.81),
            ..AssetMetaData::mock()
        };
        let balance = GemAssetBalance {
            available: BigUint::from(1_500_000u32),
            ..GemAssetBalance::mock()
        };

        assert_eq!(
            balance_rows(&asset, &metadata, &balance),
            vec![GemAssetBalanceRow {
                row: GemBalanceRow::Staked { value: BigUint::ZERO },
                value: GemBalanceRowValue::Apr {
                    apr: Some(GemFormattedNumber::percentage(15.81, GemPercentageStyle::Unsigned))
                },
            }]
        );

        let staked = GemAssetBalance {
            staked: BigUint::from(2_000_000u32),
            ..balance.clone()
        };
        assert_eq!(
            balance_rows(&asset, &metadata, &staked).into_iter().map(|item| item.value).collect::<Vec<_>>(),
            vec![
                GemBalanceRowValue::Amount {
                    amount: GemFormattedNumber::amount(1.5, Some("ATOM".to_string()), GemValueStyle::Auto)
                },
                GemBalanceRowValue::Amount {
                    amount: GemFormattedNumber::amount(2.0, Some("ATOM".to_string()), GemValueStyle::Auto)
                },
            ]
        );
        assert_eq!(
            balance_rows(&asset, &AssetMetaData { staking_apr: None, ..metadata }, &balance)[0].value,
            GemBalanceRowValue::Apr { apr: None },
            "an unknown rate leaves the label without a number"
        );
    }

    #[test]
    fn test_details_state_shows_the_header_buttons_the_metadata_allows() {
        let tradable = AssetMetaData {
            is_buy_enabled: true,
            is_swap_enabled: true,
            ..AssetMetaData::mock()
        };
        let all = state(WalletType::Multicoin, &tradable, &[]);
        assert_eq!(kinds(&all), vec![GemHeaderButtonKind::Send, GemHeaderButtonKind::Receive, GemHeaderButtonKind::Buy, GemHeaderButtonKind::Swap]);
        assert!(buttons(&all).iter().all(|button| button.is_enabled));

        let transfer_only = state(WalletType::Multicoin, &AssetMetaData::mock(), &[]);
        assert_eq!(kinds(&transfer_only), vec![GemHeaderButtonKind::Send, GemHeaderButtonKind::Receive]);
    }

    #[test]
    fn test_details_state_disables_the_buttons_behind_an_activation_or_multi_signature_banner() {
        let tradable = AssetMetaData {
            is_buy_enabled: true,
            is_swap_enabled: true,
            ..AssetMetaData::mock()
        };
        for event in [BannerEvent::ActivateAsset, BannerEvent::AccountBlockedMultiSignature] {
            let state = state(WalletType::Multicoin, &tradable, &[BannerEvent::Stake, event]);
            assert!(buttons(&state).iter().all(|button| !button.is_enabled), "{event:?}");
        }
        let stake_only = state(WalletType::Multicoin, &tradable, &[BannerEvent::Stake]);
        assert!(buttons(&stake_only).iter().all(|button| button.is_enabled));
    }

    #[test]
    fn test_details_state_hides_buttons_banners_and_earn_from_a_view_only_wallet() {
        let metadata = AssetMetaData {
            is_buy_enabled: true,
            is_swap_enabled: true,
            is_earn_enabled: true,
            ..AssetMetaData::mock()
        };
        let state = state(WalletType::View, &metadata, &[]);

        assert!(state.is_view_only);
        assert_eq!(state.header_actions, GemHeaderActions::WatchOnly);
        assert!(!state.shows_banners);
        assert_eq!(state.empty_transactions_action, Some(GemAssetEmptyAction::Buy));
    }

    #[test]
    fn test_details_state_shows_banners_for_every_wallet_type() {
        let metadata = AssetMetaData {
            is_buy_enabled: true,
            is_swap_enabled: true,
            is_earn_enabled: true,
            ..AssetMetaData::mock()
        };
        for wallet_type in [WalletType::Multicoin, WalletType::Single, WalletType::PrivateKey, WalletType::View] {
            for event in [BannerEvent::AccountBlockedMultiSignature, BannerEvent::SuspiciousAsset] {
                let state = state(wallet_type, &metadata, &[event]);
                assert!(state.shows_banners, "{wallet_type:?} {event:?}");
            }
            assert!(!state(wallet_type, &metadata, &[]).shows_banners);
        }
    }

    #[test]
    fn test_details_sections_offer_manage_until_the_balance_is_enabled() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let balance = GemAssetBalance::mock();
        let manage = |metadata: &AssetMetaData| section(&sections(&asset, metadata, &balance, Some(1.0), &[]), GemListSectionTitle::Manage);
        let link = |title: GemListRowTitle, icon: GemListRowIcon| GemAssetDetailRow::Row {
            row: GemListRow::Link { title, value: None, icon },
        };
        let unmanaged = AssetMetaData {
            is_balance_enabled: false,
            ..AssetMetaData::mock()
        };

        assert_eq!(manage(&unmanaged), vec![vec![link(GemListRowTitle::Pin, GemListRowIcon::Pin), link(GemListRowTitle::AddToWallet, GemListRowIcon::AddToWallet)]]);
        assert_eq!(manage(&AssetMetaData { is_pinned: true, ..unmanaged })[0][0], link(GemListRowTitle::Unpin, GemListRowIcon::Unpin));
        assert!(manage(&AssetMetaData::mock()).is_empty());
    }

    #[test]
    fn test_details_sections_show_resources_only_where_staking_freezes() {
        let metadata = BalanceMetadata {
            votes: 0,
            energy_available: 10,
            energy_total: 20,
            bandwidth_available: 300,
            bandwidth_total: 600,
        };
        let resources = |chain: Chain, fee_balance_metadata: Option<BalanceMetadata>| {
            let sections = details_sections(DetailsSectionsInput {
                wallet_type: WalletType::Multicoin,
                asset: &Asset::from_chain(chain),
                metadata: &AssetMetaData::mock(),
                balance: &GemAssetBalance::mock(),
                price: Some(1.0),
                price_change_percentage_24h: None,
                currency: Currency::USD,
                price_alerts: &[],
                fee_balance_metadata,
            });
            section(&sections, GemListSectionTitle::Resources)
        };
        let text = |title: GemListRowTitle, value: &str| GemAssetDetailRow::Row {
            row: GemListRow::Text { title, value: value.to_string() },
        };

        assert_eq!(
            resources(Chain::Tron, Some(metadata.clone())),
            vec![vec![text(GemListRowTitle::Energy, "10 / 20"), text(GemListRowTitle::Bandwidth, "300 / 600")]]
        );
        assert!(resources(Chain::Tron, None).is_empty(), "a resource section needs the fee asset's resources");
        assert!(resources(Chain::Cosmos, Some(metadata.clone())).is_empty());
        assert!(resources(Chain::Bitcoin, Some(metadata)).is_empty());
    }

    #[test]
    fn test_a_price_row_carries_the_quote_and_hides_one_nobody_gave() {
        let quoted = price_row(Some(1234.5), Some(-2.5), Currency::USD);
        let GemPriceRow { price, change } = quoted;
        let price = price.expect("a quoted price is shown");
        let change = change.expect("a change is shown beside it");

        assert_eq!(price.value, 1234.5);
        assert_eq!(price.unit, crate::formatted_number::GemNumberUnit::Currency { code: "USD".to_string() });
        assert_eq!(change.tone, crate::formatted_number::GemValueTone::Negative, "a falling price reads red on both apps");

        let unquoted = price_row(Some(0.0), None, Currency::USD);
        assert_eq!(unquoted, GemPriceRow { price: None, change: None }, "neither app has to decide what a zero price reads as");
        assert_eq!(price_row(None, None, Currency::USD), GemPriceRow { price: None, change: None });
    }

    #[test]
    fn test_the_earn_row_reads_its_apr_the_same_way_stake_does() {
        let earn_enabled = AssetMetaData {
            is_earn_enabled: true,
            earn_apr: Some(4.0),
            ..AssetMetaData::mock()
        };
        let rows = sections(&Asset::from_chain(Chain::Ethereum), &earn_enabled, &GemAssetBalance::mock(), Some(1.0), &[]);
        let earn = rows.iter().flat_map(|section| section.rows.clone()).find_map(|row| match row {
            GemAssetDetailRow::Row { row } if matches!(row, GemListRow::Amount { title: GemListRowTitle::StakeApr, .. } | GemListRow::Text { title: GemListRowTitle::StakeApr, .. }) => Some(row),
            _ => None,
        });

        assert_eq!(earn, EARN_OFFERED.then(|| crate::services::stake::rules::earn_apr_row(&[], Some(4.0))));
    }

    #[test]
    fn test_details_sections_list_price_network_and_balances_in_order() {
        let token = Asset::mock_ethereum_usdc();
        let reserving = GemAssetBalance {
            reserved: GemBigUint::from(10u32),
            ..GemAssetBalance::mock_with_available(100)
        };
        let sections = sections(&token, &AssetMetaData::mock(), &reserving, Some(1.0), &[]);

        assert_eq!(sections.iter().map(|section| section.title).collect::<Vec<_>>(), vec![GemListSectionTitle::None, GemListSectionTitle::Balances]);
        assert!(matches!(
            sections[0].rows[0],
            GemAssetDetailRow::Row {
                row: GemListRow::Quote { title: GemListRowTitle::Price, .. }
            }
        ));
        assert_eq!(
            sections[0].rows[1],
            GemAssetDetailRow::Row {
                row: GemListRow::Network {
                    title: GemListRowTitle::Network,
                    chain: Chain::Ethereum,
                    name: "Ethereum (ERC20)".to_string(),
                },
            }
        );
        assert!(sections[1].rows.iter().all(|row| matches!(row, GemAssetDetailRow::Balance { .. })));
    }

    #[test]
    fn test_the_header_fiat_value_counts_the_whole_balance_and_hides_without_value() {
        let usdc = Asset::mock_ethereum_usdc();
        let balance = GemAssetBalance {
            staked: GemBigUint::from(500_000u32),
            reserved: GemBigUint::from(9_000_000u32),
            ..GemAssetBalance::mock_with_available(1_000_000)
        };

        assert_eq!(fiat_value(&usdc, &balance, Some(2.0), Currency::EUR), Some(GemFormattedNumber::currency(3.0, Currency::EUR, GemCurrencyStyle::Currency)));
        assert_eq!(fiat_value(&usdc, &balance, None, Currency::USD), None);
        assert_eq!(fiat_value(&usdc, &balance, Some(0.0), Currency::USD), None);
        assert_eq!(fiat_value(&usdc, &GemAssetBalance::mock(), Some(2.0), Currency::USD), None);
    }

    #[test]
    fn test_a_native_asset_is_titled_by_its_chain() {
        let mut stale = Asset::from_chain(Chain::Ethereum);
        stale.name = "Ether (old)".to_string();
        assert_eq!(asset_title(&stale), "Ethereum", "a native row reads the chain's own name, not the stored one");

        let token = Asset::mock_ethereum_usdc();
        assert_eq!(asset_title(&token), token.name, "a token keeps the name it was stored with");
    }

    #[test]
    fn test_details_sections_count_displayed_price_alerts_only_with_a_price() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let metadata = AssetMetaData::mock();
        let balance = GemAssetBalance::mock();
        let auto = PriceAlert::new_auto(AssetId::from_chain(Chain::Ethereum), Currency::USD);
        let manual = PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 120.0, PriceAlertDirection::Up);
        let mut notified = PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 140.0, PriceAlertDirection::Up);
        notified.last_notified_at = Some(Utc::now());
        let alerts_row = |price: Option<f64>, alerts: Vec<PriceAlert>| {
            sections(&asset, &metadata, &balance, price, &alerts)[0].rows.iter().find_map(|row| match row {
                GemAssetDetailRow::Row {
                    row: GemListRow::Link {
                        title: GemListRowTitle::PriceAlerts, value, ..
                    },
                } => value.clone(),
                _ => None,
            })
        };

        assert_eq!(alerts_row(Some(1.0), vec![auto.clone(), manual, notified.clone()]), Some("2".to_string()));
        assert_eq!(alerts_row(Some(1.0), vec![notified]), None);
        assert_eq!(alerts_row(Some(0.0), vec![auto.clone()]), None);
        assert_eq!(alerts_row(None, vec![auto]), None);
        assert_eq!(alerts_row(Some(1.0), vec![]), None);
    }

    #[test]
    fn test_applied_filters_add_the_picked_chains_and_the_balance_only_where_the_flow_offers_it() {
        let send = GemSelectAssetType::Send.flow();
        let receive = GemSelectAssetType::Receive.flow();
        let chains = vec![Chain::Ethereum, Chain::Solana];

        assert_eq!(send.applied_filters(vec![], false), send.filters);
        assert_eq!(
            receive.applied_filters(chains.clone(), true),
            [
                receive.filters.clone(),
                vec![GemAssetFilter::Chains { chains: chains.clone() }],
                receive.balance_filter.then_some(GemAssetFilter::HasBalance).into_iter().collect(),
            ]
            .concat()
        );
        assert_eq!(
            send.applied_filters(vec![], true).iter().filter(|filter| **filter == GemAssetFilter::HasBalance).count(),
            1,
            "a flow that already filters by balance does not add it twice"
        );
    }

    #[test]
    fn test_details_state_reports_the_auto_alert() {
        let plain = AssetMetaData::mock();
        let auto = PriceAlert::new_auto(AssetId::from_chain(Chain::Ethereum), Currency::USD);
        let manual = PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 120.0, PriceAlertDirection::Up);
        let mut notified = PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), Currency::USD, 140.0, PriceAlertDirection::Up);
        notified.last_notified_at = Some(Utc::now());
        let state = |alerts: Vec<PriceAlert>| details_state(WalletType::Multicoin, &plain, &[], &alerts);

        assert_eq!(state(vec![auto.clone(), manual.clone(), notified]).price_alert, GemPriceAlertToggle::Enabled);
        assert_eq!(state(vec![manual]).price_alert, GemPriceAlertToggle::Disabled);
    }

    #[test]
    fn test_every_select_flow_names_its_own_screen() {
        assert_eq!(select_asset_flow(GemSelectAssetType::Manage, None).title, GemSelectAssetTitle::ManageTokenList);
        assert_eq!(select_asset_flow(GemSelectAssetType::SwapPay, None).title, GemSelectAssetTitle::SwapPay);
        assert_eq!(select_asset_flow(GemSelectAssetType::SwapReceive { pay_asset_id: None }, None).title, GemSelectAssetTitle::SwapReceive);
        assert_eq!(select_asset_flow(GemSelectAssetType::PriceAlert, None).title, GemSelectAssetTitle::SelectAsset);
        assert_eq!(
            select_asset_flow(GemSelectAssetType::ReceiveCollection, None).assets_section,
            GemSelectAssetSection::Networks,
            "a collection is received on a network, not on an asset"
        );
        assert_eq!(select_asset_flow(GemSelectAssetType::Receive, None).assets_section, GemSelectAssetSection::Assets);
    }

    #[test]
    fn test_recents_show_only_outside_a_search_and_when_there_are_some() {
        let flow = select_asset_flow(GemSelectAssetType::Send, None);

        assert!(flow.shows_recents(false, true));
        assert!(!flow.shows_recents(true, true));
        assert!(!flow.shows_recents(false, false));
    }

    #[test]
    fn test_add_token_and_the_chain_filter_need_chains_to_act_on() {
        let flow = select_asset_flow(GemSelectAssetType::Send, None);
        let manage = select_asset_flow(GemSelectAssetType::Manage, None);

        assert!(manage.add_custom_token && manage.shows_add_token(true, true));
        assert!(!manage.shows_add_token(false, true), "a wallet that holds no tokens cannot add one");
        assert!(!manage.shows_add_token(true, false), "there is nothing to add a token to without a chain");
        assert!(!flow.shows_chain_filter(true, true) || flow.chain_filter);
        assert!(!manage.shows_chain_filter(false, true), "a single chain wallet has nothing to filter");
        assert!(!manage.shows_chain_filter(true, false));
    }

    #[test]
    fn test_details_sections_offer_earn_until_there_is_an_earn_balance() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let earn_enabled = AssetMetaData {
            is_earn_enabled: true,
            earn_apr: Some(4.0),
            ..AssetMetaData::mock()
        };
        let earning = GemAssetBalance {
            earn: GemBigUint::from(100u32),
            ..GemAssetBalance::mock()
        };
        let offers_earn = |metadata: &AssetMetaData, balance: &GemAssetBalance| {
            sections(&asset, metadata, balance, Some(1.0), &[]).iter().flat_map(|section| section.rows.clone()).any(|row| {
                matches!(
                    row,
                    GemAssetDetailRow::Row {
                        row: GemListRow::Amount { title: GemListRowTitle::StakeApr, .. } | GemListRow::Text { title: GemListRowTitle::StakeApr, .. }
                    }
                )
            })
        };

        assert_eq!(offers_earn(&earn_enabled, &GemAssetBalance::mock()), EARN_OFFERED);
        assert!(!offers_earn(&earn_enabled, &earning));
        assert!(!offers_earn(&AssetMetaData::mock(), &GemAssetBalance::mock()));
    }

    #[test]
    fn test_details_state_empty_transactions_prefer_buy_then_swap() {
        let swappable = AssetMetaData {
            is_swap_enabled: true,
            ..AssetMetaData::mock()
        };
        let tradable = AssetMetaData { is_buy_enabled: true, ..swappable.clone() };
        assert_eq!(state(WalletType::Multicoin, &tradable, &[]).empty_transactions_action, Some(GemAssetEmptyAction::Buy));
        assert_eq!(state(WalletType::Multicoin, &swappable, &[]).empty_transactions_action, Some(GemAssetEmptyAction::Swap));
        assert_eq!(state(WalletType::Multicoin, &AssetMetaData::mock(), &[]).empty_transactions_action, None);
    }
}
