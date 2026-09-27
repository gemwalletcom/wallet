#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemEmptyStateKind {
    Nfts,
    PriceAlerts,
    Contacts,
    Asset,
    Activity,
    Stake,
    Earn,
    Validators,
    WalletConnect,
    Recents,
    Notifications,
    NetworkAssets,
    SearchAssets,
    SearchNetworks,
    SearchActivity,
    SearchPerpetuals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemEmptyStateAction {
    Buy,
    Swap,
    Receive,
    AddCustomToken,
    ManageTokenList,
    ClearFilters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemEmptyStateText {
    NftsTitle,
    NftsDescription,
    PriceAlertsTitle,
    PriceAlertsDescription,
    ContactsTitle,
    ContactsDescription,
    AssetTitle,
    AssetDescription,
    ActivityTitle,
    ActivityDescription,
    StakeTitle,
    StakeDescription,
    EarnTitle,
    EarnDescription,
    ValidatorsTitle,
    WalletConnectTitle,
    WalletConnectDescription,
    RecentsTitle,
    RecentsDescription,
    NotificationsTitle,
    NotificationsDescription,
    WatchWalletTitle,
    WatchWalletDescription,
    NoAssetsFoundTitle,
    SearchDescription,
    SearchAssetsDescription,
    SearchActivityTitle,
    SearchActivityDescription,
    SearchNetworksTitle,
    SearchPerpetualsTitle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemEmptyStateImage {
    Nfts,
    PriceAlerts,
    Contacts,
    Activity,
    Stake,
    WalletConnect,
    Notifications,
    Search,
    Wallet,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemEmptyState {
    pub title: GemEmptyStateText,
    pub description: Option<GemEmptyStateText>,
    pub image: GemEmptyStateImage,
    pub actions: Vec<GemEmptyStateAction>,
}

/// The empty state of a screen whose actions depend on nothing but its kind.
#[uniffi::export]
pub fn empty_state(kind: GemEmptyStateKind) -> GemEmptyState {
    screen_empty_state(kind, false, &[GemEmptyStateAction::ManageTokenList, GemEmptyStateAction::ClearFilters])
}

pub(crate) fn screen_empty_state(kind: GemEmptyStateKind, is_view_only: bool, offered_actions: &[GemEmptyStateAction]) -> GemEmptyState {
    use GemEmptyStateAction::*;
    use GemEmptyStateKind::*;
    use GemEmptyStateText::*;

    let offers = |action: GemEmptyStateAction| offered_actions.contains(&action);
    let offered = |actions: &[GemEmptyStateAction]| actions.iter().copied().filter(|action| offers(*action)).collect();
    let watch_only = |title, description, image| GemEmptyState {
        title: if is_view_only { WatchWalletTitle } else { title },
        description: Some(if is_view_only { WatchWalletDescription } else { description }),
        image,
        actions: if is_view_only { vec![] } else { offered(&[Buy, Receive, Swap]) },
    };

    match kind {
        Nfts => GemEmptyState {
            title: NftsTitle,
            description: offers(Receive).then_some(NftsDescription),
            image: GemEmptyStateImage::Nfts,
            actions: offered(&[Receive]),
        },
        PriceAlerts => GemEmptyState {
            title: PriceAlertsTitle,
            description: Some(PriceAlertsDescription),
            image: GemEmptyStateImage::PriceAlerts,
            actions: vec![],
        },
        Contacts => GemEmptyState {
            title: ContactsTitle,
            description: Some(ContactsDescription),
            image: GemEmptyStateImage::Contacts,
            actions: vec![],
        },
        Asset => watch_only(AssetTitle, AssetDescription, GemEmptyStateImage::Activity),
        Activity => watch_only(ActivityTitle, ActivityDescription, GemEmptyStateImage::Activity),
        Stake => GemEmptyState {
            title: StakeTitle,
            description: Some(StakeDescription),
            image: GemEmptyStateImage::Stake,
            actions: vec![],
        },
        Earn => GemEmptyState {
            title: EarnTitle,
            description: Some(EarnDescription),
            image: GemEmptyStateImage::Stake,
            actions: vec![],
        },
        Validators => GemEmptyState {
            title: ValidatorsTitle,
            description: None,
            image: GemEmptyStateImage::Stake,
            actions: vec![],
        },
        WalletConnect => GemEmptyState {
            title: WalletConnectTitle,
            description: Some(WalletConnectDescription),
            image: GemEmptyStateImage::WalletConnect,
            actions: vec![],
        },
        Recents => GemEmptyState {
            title: RecentsTitle,
            description: Some(RecentsDescription),
            image: GemEmptyStateImage::Activity,
            actions: vec![],
        },
        Notifications => GemEmptyState {
            title: NotificationsTitle,
            description: Some(NotificationsDescription),
            image: GemEmptyStateImage::Notifications,
            actions: vec![],
        },
        NetworkAssets => GemEmptyState {
            title: NoAssetsFoundTitle,
            description: None,
            image: GemEmptyStateImage::Wallet,
            actions: offered(&[ManageTokenList]),
        },
        SearchAssets => GemEmptyState {
            title: NoAssetsFoundTitle,
            description: Some(if offers(AddCustomToken) { SearchAssetsDescription } else { SearchDescription }),
            image: GemEmptyStateImage::Search,
            actions: offered(&[AddCustomToken]),
        },
        SearchNetworks => GemEmptyState {
            title: SearchNetworksTitle,
            description: Some(SearchDescription),
            image: GemEmptyStateImage::Search,
            actions: vec![],
        },
        SearchActivity => GemEmptyState {
            title: SearchActivityTitle,
            description: Some(SearchActivityDescription),
            image: GemEmptyStateImage::Search,
            actions: offered(&[ClearFilters]),
        },
        SearchPerpetuals => GemEmptyState {
            title: SearchPerpetualsTitle,
            description: Some(SearchDescription),
            image: GemEmptyStateImage::Search,
            actions: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_watch_only_list_explains_itself_and_offers_nothing() {
        let watching = screen_empty_state(GemEmptyStateKind::Activity, true, &[GemEmptyStateAction::Buy]);
        assert_eq!(watching.title, GemEmptyStateText::WatchWalletTitle);
        assert_eq!(watching.description, Some(GemEmptyStateText::WatchWalletDescription));
        assert!(watching.actions.is_empty());

        let owned = screen_empty_state(GemEmptyStateKind::Activity, false, &[GemEmptyStateAction::Buy, GemEmptyStateAction::Receive]);
        assert_eq!(owned.title, GemEmptyStateText::ActivityTitle);
        assert_eq!(owned.actions, vec![GemEmptyStateAction::Buy, GemEmptyStateAction::Receive]);
    }

    #[test]
    fn test_only_the_actions_the_screen_offers_are_returned() {
        assert_eq!(screen_empty_state(GemEmptyStateKind::Asset, false, &[GemEmptyStateAction::Swap]).actions, vec![GemEmptyStateAction::Swap]);
        assert!(screen_empty_state(GemEmptyStateKind::Asset, false, &[]).actions.is_empty());
        assert!(screen_empty_state(GemEmptyStateKind::Nfts, false, &[]).description.is_none(), "an nft list without a receive action says only that it is empty");
        assert_eq!(screen_empty_state(GemEmptyStateKind::Nfts, false, &[GemEmptyStateAction::Receive]).description, Some(GemEmptyStateText::NftsDescription));
    }

    #[test]
    fn test_a_kind_alone_offers_only_the_actions_that_need_nothing_else() {
        assert_eq!(empty_state(GemEmptyStateKind::NetworkAssets).actions, vec![GemEmptyStateAction::ManageTokenList]);
        assert!(empty_state(GemEmptyStateKind::Nfts).actions.is_empty(), "receiving needs a screen that can receive");
        assert!(empty_state(GemEmptyStateKind::Activity).actions.is_empty());
    }

    #[test]
    fn test_an_empty_validator_list_says_so_instead_of_failing() {
        let state = screen_empty_state(GemEmptyStateKind::Validators, false, &[GemEmptyStateAction::Buy]);
        assert_eq!(state.title, GemEmptyStateText::ValidatorsTitle);
        assert_eq!(state.description, None);
        assert_eq!(state.image, GemEmptyStateImage::Stake);
        assert!(state.actions.is_empty());
    }

    #[test]
    fn test_a_search_without_a_custom_token_action_falls_back_to_the_plain_description() {
        assert_eq!(screen_empty_state(GemEmptyStateKind::SearchAssets, false, &[]).description, Some(GemEmptyStateText::SearchDescription));
        assert_eq!(
            screen_empty_state(GemEmptyStateKind::SearchAssets, false, &[GemEmptyStateAction::AddCustomToken]).description,
            Some(GemEmptyStateText::SearchAssetsDescription)
        );
    }
}
