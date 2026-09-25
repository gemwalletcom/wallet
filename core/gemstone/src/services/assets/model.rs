use primitives::{Asset, AssetId, AssetMetaData, AssetType, BalanceMetadata, Banner, BlockExplorerLink, Chain, Currency, PriceAlert, RecentActivityType, VerificationStatus, Wallet};

use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemListSectionTitle};
use crate::precision::GemCurrencyStyle;
use crate::services::balance::{GemAssetBalance, GemAssetBalanceRow};
use crate::services::banner::GemBannerRow;
use crate::services::price_alert::rules::GemPriceAlertToggle;
use crate::services::swap::GemSwapPairSuggestion;
use strum::IntoEnumIterator;
use swapper::AssetList as SwapAssetList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetList {
    Buy,
    Sell,
    Swap,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetNetworkDestination {
    Asset { asset: Asset },
    Assets { chain: Chain },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetAction {
    Open,
    Send,
    Receive,
    Buy,
    Sell,
    SwapPay,
    SwapReceive,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectAssetType {
    Send,
    Receive,
    ReceiveCollection,
    Buy,
    SwapPay,
    SwapReceive { pay_asset_id: Option<AssetId> },
    Payment { asset_ids: Vec<AssetId> },
    Manage,
    PriceAlert,
    Deposit,
    Withdraw,
    WalletSearch,
    WalletSearchResults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectRowAction {
    Navigate,
    Toggle,
    Select,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectAssetScope {
    Wallet,
    AllAssets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetTitleStyle {
    Asset,
    CanonicalAsset,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetSubtitleStyle {
    Network,
    Price,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetTrailingStyle {
    Balance,
    Toggle,
    Copy,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetText {
    pub title: String,
    pub subtitle_symbol: Option<String>,
    pub network_name: String,
    pub network_full_name: String,
}

#[uniffi::export]
pub fn asset_text(asset: Asset) -> GemAssetText {
    super::rules::asset_text(&asset)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetRowText {
    pub title: String,
    pub symbol: Option<String>,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetBalanceScope {
    Total,
    Available,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetListRowInput {
    pub asset: Asset,
    pub balance: GemAssetBalance,
    pub scope: GemAssetBalanceScope,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub currency: Currency,
    pub style: GemAssetRowStyle,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceRow {
    pub price: Option<GemFormattedNumber>,
    pub change: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetListRow {
    pub icon: super::icon::GemAssetIcon,
    pub text: GemAssetRowText,
    pub price: GemPriceRow,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
}

#[uniffi::export]
pub fn asset_list_row(input: GemAssetListRowInput) -> GemAssetListRow {
    super::rules::asset_list_row(input)
}

#[uniffi::export]
pub fn asset_list_rows(inputs: Vec<GemAssetListRowInput>) -> Vec<GemAssetListRow> {
    inputs.into_iter().map(super::rules::asset_list_row).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetRowStyle {
    pub title: GemAssetTitleStyle,
    pub shows_symbol: bool,
    pub subtitle: GemAssetSubtitleStyle,
    pub trailing: GemAssetTrailingStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectAssetTitle {
    Send,
    PayWith,
    Receive,
    ReceiveCollection,
    Buy,
    SwapPay,
    SwapReceive,
    ManageTokenList,
    SelectAsset,
    Deposit,
    Withdraw,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectAssetSection {
    Assets,
    Networks,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSelectAssetFlow {
    pub title: GemSelectAssetTitle,
    pub assets_section: GemSelectAssetSection,
    pub row_style: GemAssetRowStyle,
    pub row_action: GemSelectRowAction,
    pub action: Option<GemAssetAction>,
    pub scope: GemSelectAssetScope,
    pub filters: Vec<GemAssetFilter>,
    pub enables_price_alert: bool,
    pub network_search: bool,
    pub chain_filter: bool,
    pub recents: bool,
    pub popular_section: bool,
    pub balance_filter: bool,
    pub add_custom_token: bool,
    pub display_asset: Option<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetSearchStep {
    Idle,
    Search { query: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSelectAssetState {
    Idle,
    Loading,
    Empty,
}

#[uniffi::export]
impl GemSelectAssetFlow {
    pub fn shows_recents(&self, is_searching: bool, has_recents: bool) -> bool {
        self.recents && !is_searching && has_recents
    }

    pub fn search_step(&self, query: String) -> GemAssetSearchStep {
        let query = query.trim();
        match self.network_search && !query.is_empty() {
            true => GemAssetSearchStep::Search { query: query.to_string() },
            false => GemAssetSearchStep::Idle,
        }
    }

    pub fn state(&self, counts: GemAssetSectionCounts, is_searching: bool) -> GemSelectAssetState {
        match (counts.pinned + counts.popular + counts.assets > 0, is_searching) {
            (true, _) => GemSelectAssetState::Idle,
            (false, true) => GemSelectAssetState::Loading,
            (false, false) => GemSelectAssetState::Empty,
        }
    }

    pub fn applied_filters(&self, chains: Vec<Chain>, has_balance: bool) -> Vec<GemAssetFilter> {
        super::rules::applied_filters(self, chains, has_balance)
    }
}

impl GemSelectAssetFlow {
    pub fn shows_add_token(&self, supports_tokens: bool, has_chains: bool) -> bool {
        self.add_custom_token && supports_tokens && has_chains
    }

    pub fn shows_chain_filter(&self, is_multicoin: bool, has_chains: bool) -> bool {
        self.chain_filter && is_multicoin && has_chains
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSelectAssetWalletFlow {
    pub flow: GemSelectAssetFlow,
    pub chains: Vec<Chain>,
    pub shows_add_token: bool,
    pub shows_chain_filter: bool,
}

#[uniffi::export]
impl GemSelectAssetType {
    pub fn flow(&self) -> GemSelectAssetFlow {
        super::rules::select_asset_flow(self.clone(), None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetFilter {
    Enabled,
    Buyable,
    Sellable,
    Swappable,
    HasBalance,
    HasAvailableBalance,
    ChainsOrAssetIds { chains: Vec<Chain>, asset_ids: Vec<AssetId> },
    Chains { chains: Vec<Chain> },
}

impl GemAssetFilter {
    pub fn asset_ids(asset_ids: Vec<AssetId>) -> Self {
        Self::ChainsOrAssetIds { chains: Vec::new(), asset_ids }
    }
}

impl From<SwapAssetList> for GemAssetFilter {
    fn from(assets: SwapAssetList) -> Self {
        Self::ChainsOrAssetIds {
            chains: assets.chains,
            asset_ids: assets.asset_ids,
        }
    }
}

#[uniffi::export]
impl GemAssetAction {
    pub fn recent_activity_types(&self) -> Vec<RecentActivityType> {
        match self {
            Self::SwapPay | Self::SwapReceive => vec![RecentActivityType::SwapSelect, RecentActivityType::Swap],
            Self::Open | Self::Send | Self::Receive | Self::Buy | Self::Sell => RecentActivityType::iter().collect(),
        }
    }
}

impl GemAssetAction {
    pub fn filters(&self) -> Vec<GemAssetFilter> {
        match self {
            Self::Open => Vec::new(),
            Self::Send => vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance],
            Self::Receive => vec![GemAssetFilter::Enabled],
            Self::Buy => vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable],
            Self::Sell => vec![GemAssetFilter::Enabled, GemAssetFilter::Sellable],
            Self::SwapPay => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance],
            Self::SwapReceive => vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable],
        }
    }

    pub fn recent_activity_type(&self, asset: &Asset) -> Option<RecentActivityType> {
        match self {
            Self::Open => Some(match asset.asset_type {
                AssetType::PERPETUAL => RecentActivityType::Perpetual,
                _ => RecentActivityType::Search,
            }),
            Self::Send => None,
            Self::Receive => Some(RecentActivityType::Receive),
            Self::Buy => Some(RecentActivityType::FiatBuy),
            Self::Sell => Some(RecentActivityType::FiatSell),
            Self::SwapPay | Self::SwapReceive => Some(RecentActivityType::SwapSelect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Asset, AssetType, GemAssetAction, GemAssetFilter, GemAssetSearchStep, GemAssetSectionCounts, GemSelectAssetState, GemSelectAssetType, RecentActivityType};
    use primitives::Chain;

    #[test]
    fn test_a_search_runs_only_on_a_trimmed_query_a_network_flow_accepts() {
        let network = GemSelectAssetType::Buy.flow();
        assert!(network.network_search);
        assert_eq!(network.search_step("  btc ".to_string()), GemAssetSearchStep::Search { query: "btc".to_string() });
        assert_eq!(network.search_step("   ".to_string()), GemAssetSearchStep::Idle);
        assert_eq!(network.search_step(String::new()), GemAssetSearchStep::Idle);

        let local = GemSelectAssetType::Deposit.flow();
        assert!(!local.network_search);
        assert_eq!(local.search_step("btc".to_string()), GemAssetSearchStep::Idle);
    }

    #[test]
    fn test_the_list_reads_as_loading_only_while_a_search_finds_nothing() {
        let flow = GemSelectAssetType::Buy.flow();
        let listed = GemAssetSectionCounts { assets: 2, ..Default::default() };
        let none = GemAssetSectionCounts::default();
        assert_eq!(flow.state(listed, true), GemSelectAssetState::Idle);
        assert_eq!(flow.state(listed, false), GemSelectAssetState::Idle);
        assert_eq!(flow.state(none, true), GemSelectAssetState::Loading);
        assert_eq!(flow.state(none, false), GemSelectAssetState::Empty);
    }

    #[test]
    fn test_a_result_of_only_popular_or_pinned_rows_is_a_list() {
        let flow = GemSelectAssetType::Buy.flow();
        assert_eq!(flow.state(GemAssetSectionCounts { popular: 3, ..Default::default() }, false), GemSelectAssetState::Idle);
        assert_eq!(flow.state(GemAssetSectionCounts { pinned: 1, ..Default::default() }, true), GemSelectAssetState::Idle);
    }

    #[test]
    fn test_every_recorded_recent_type_is_shown_by_the_same_action() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let perpetual = Asset {
            asset_type: AssetType::PERPETUAL,
            ..Asset::from_chain(Chain::HyperCore)
        };
        let actions = [
            GemAssetAction::Open,
            GemAssetAction::Send,
            GemAssetAction::Receive,
            GemAssetAction::Buy,
            GemAssetAction::Sell,
            GemAssetAction::SwapPay,
            GemAssetAction::SwapReceive,
        ];
        for action in actions {
            for asset in [&asset, &perpetual] {
                if let Some(recorded) = action.recent_activity_type(asset) {
                    assert!(action.recent_activity_types().contains(&recorded), "{action:?}");
                }
            }
        }
        assert_eq!(GemAssetAction::Send.recent_activity_type(&asset), None);
        assert_eq!(GemAssetAction::Open.recent_activity_type(&perpetual), Some(RecentActivityType::Perpetual));
        assert_eq!(GemAssetAction::Open.recent_activity_type(&asset), Some(RecentActivityType::Search));
        assert!(!GemAssetAction::SwapPay.recent_activity_types().contains(&RecentActivityType::Receive));
    }

    #[test]
    fn test_action_filters_gate_on_the_balance_each_action_can_spend() {
        assert_eq!(GemAssetAction::Send.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::HasBalance]);
        assert_eq!(GemAssetAction::SwapPay.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable, GemAssetFilter::HasAvailableBalance]);
        assert_eq!(GemAssetAction::SwapReceive.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Swappable]);
        assert_eq!(GemAssetAction::Buy.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Buyable]);
        assert_eq!(GemAssetAction::Sell.filters(), vec![GemAssetFilter::Enabled, GemAssetFilter::Sellable]);
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletSearchLimits {
    pub assets: u32,
    pub fetch: u32,
    pub perpetuals: u32,
    pub nfts: u32,
    pub results: u32,
}

impl GemWalletSearchLimits {
    pub fn has_more_assets(&self, count: u32) -> bool {
        count > self.assets
    }

    pub fn has_more_perpetuals(&self, count: u32) -> bool {
        count > self.perpetuals
    }

    pub fn has_more_nfts(&self, count: u32) -> bool {
        count > self.nfts
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetSectionIds {
    pub pinned: Vec<AssetId>,
    pub popular: Vec<AssetId>,
    pub assets: Vec<AssetId>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetSectionCounts {
    pub pinned: u32,
    pub popular: u32,
    pub assets: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemNetworkAssetSections {
    pub shows_pinned: bool,
    pub shows_unpinned: bool,
    pub shows_hidden: bool,
    pub shows_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemNetworkAssetIds {
    pub pinned: Vec<AssetId>,
    pub unpinned: Vec<AssetId>,
    pub hidden: Vec<AssetId>,
    pub sections: GemNetworkAssetSections,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFeeAmount {
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
}

#[uniffi::export]
pub fn fee_amount(asset: Asset, value: GemBigInt, price: Option<f64>, currency: Currency) -> GemFeeAmount {
    super::rules::fee_amount(&asset, &value, price, currency)
}

#[uniffi::export]
pub fn fiat_equivalent(asset: Asset, value: GemBigInt, price: Option<f64>, currency: Currency) -> Option<GemFormattedNumber> {
    let value = value.to_biguint()?;
    super::rules::fiat_amount_of(&asset, &value, price, currency, GemCurrencyStyle::Currency)
}

#[uniffi::export]
pub fn network_asset_sections(active: Vec<AssetId>, pinned: Vec<AssetId>, hidden: Vec<AssetId>) -> GemNetworkAssetIds {
    super::rules::network_asset_sections(active, &pinned, hidden)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemHeaderButtonKind {
    Send,
    Receive,
    Buy,
    Swap,
    Deposit,
    Withdraw,
    More,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemWalletSearchCounts {
    pub recents: u32,
    pub pinned_assets: u32,
    pub assets: u32,
    pub pinned_perpetuals: u32,
    pub perpetuals: u32,
    pub lists: u32,
    pub nfts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemWalletSearchState {
    pub phase: GemSelectAssetState,
    pub shows_recents: bool,
    pub shows_pinned: bool,
    pub shows_assets: bool,
    pub shows_pinned_perpetuals: bool,
    pub shows_perpetuals: bool,
    pub shows_lists: bool,
    pub shows_nfts: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletSearchInput {
    pub wallet: Wallet,
    pub query: String,
    pub is_loading: bool,
    pub counts: GemWalletSearchCounts,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletSearchView {
    pub state: GemWalletSearchState,
    pub limits: GemWalletSearchLimits,
    pub has_more_assets: bool,
    pub has_more_perpetuals: bool,
    pub has_more_nfts: bool,
    pub shows_add_token: bool,
}

#[uniffi::export]
pub fn wallet_search_state(counts: GemWalletSearchCounts, is_loading: bool) -> GemWalletSearchState {
    super::rules::wallet_search_state(&counts, is_loading)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetMenuAction {
    Pin { is_pinned: bool },
    Hide,
    AddToWallet,
    CopyAddress { address: String },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetMenuInput {
    pub is_pinned: bool,
    pub is_balance_enabled: bool,
    pub address: String,
    pub offers_hide: bool,
    pub offers_add_to_wallet: bool,
}

#[uniffi::export]
pub fn asset_menu_actions(input: GemAssetMenuInput) -> Vec<GemAssetMenuAction> {
    super::rules::menu_actions(&input)
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemHeaderButton {
    pub kind: GemHeaderButtonKind,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemHeaderActions {
    WatchOnly,
    Buttons { buttons: Vec<GemHeaderButton> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetEmptyAction {
    Buy,
    Swap,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetDetailsState {
    pub is_view_only: bool,
    pub header_actions: GemHeaderActions,
    pub shows_banners: bool,
    pub price_alert: GemPriceAlertToggle,
    pub empty_transactions_action: Option<GemAssetEmptyAction>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetDetailRow {
    Balance { row: GemAssetBalanceRow },
    Row { row: GemListRow },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetDetailSection {
    pub title: GemListSectionTitle,
    pub rows: Vec<GemAssetDetailRow>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetDetailsInput {
    pub wallet: Wallet,
    pub asset: Asset,
    pub owner_address: Option<String>,
    pub metadata: AssetMetaData,
    pub balance: GemAssetBalance,
    pub price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub currency: Currency,
    pub banners: Vec<Banner>,
    pub price_alerts: Vec<PriceAlert>,
    pub fee_balance_metadata: Option<BalanceMetadata>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAssetDetails {
    pub state: GemAssetDetailsState,
    pub banner: Option<GemBannerRow>,
    pub balance_value: GemFormattedNumber,
    pub sections: Vec<GemAssetDetailSection>,
    pub title: String,
    pub fiat_value: Option<GemFormattedNumber>,
    pub explorer_name: String,
    pub address_link: Option<BlockExplorerLink>,
    pub token_link: Option<BlockExplorerLink>,
    pub verification_status: Option<VerificationStatus>,
    pub network_destination: Option<GemAssetNetworkDestination>,
    pub share_url: String,
    pub swap_pair: GemSwapPairSuggestion,
}
