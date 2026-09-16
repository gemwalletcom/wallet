#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemEmptyStateKind {
    Nfts,
    PriceAlerts,
    Contacts,
    Asset,
    Activity,
    Stake,
    Earn,
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
pub struct GemEmptyStateInput {
    pub kind: GemEmptyStateKind,
    pub is_view_only: bool,
    pub offered_actions: Vec<GemEmptyStateAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemEmptyState {
    pub title: GemEmptyStateText,
    pub description: Option<GemEmptyStateText>,
    pub image: GemEmptyStateImage,
    pub actions: Vec<GemEmptyStateAction>,
}

#[uniffi::export]
pub fn empty_state(input: GemEmptyStateInput) -> GemEmptyState {
    use GemEmptyStateAction::*;
    use GemEmptyStateKind::*;
    use GemEmptyStateText::*;

    let offers = |action: GemEmptyStateAction| input.offered_actions.contains(&action);
    let offered = |actions: &[GemEmptyStateAction]| actions.iter().copied().filter(|action| offers(*action)).collect();
    let watch_only = |title, description, image| GemEmptyState {
        title: if input.is_view_only { WatchWalletTitle } else { title },
        description: Some(if input.is_view_only { WatchWalletDescription } else { description }),
        image,
        actions: if input.is_view_only { vec![] } else { offered(&[Buy, Receive, Swap]) },
    };

    match input.kind {
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
        let watching = empty_state(GemEmptyStateInput {
            kind: GemEmptyStateKind::Activity,
            is_view_only: true,
            offered_actions: vec![GemEmptyStateAction::Buy],
        });
        assert_eq!(watching.title, GemEmptyStateText::WatchWalletTitle);
        assert_eq!(watching.description, Some(GemEmptyStateText::WatchWalletDescription));
        assert!(watching.actions.is_empty());

        let owned = empty_state(GemEmptyStateInput {
            kind: GemEmptyStateKind::Activity,
            is_view_only: false,
            offered_actions: vec![GemEmptyStateAction::Buy, GemEmptyStateAction::Receive],
        });
        assert_eq!(owned.title, GemEmptyStateText::ActivityTitle);
        assert_eq!(owned.actions, vec![GemEmptyStateAction::Buy, GemEmptyStateAction::Receive]);
    }

    #[test]
    fn test_only_the_actions_the_screen_offers_are_returned() {
        assert_eq!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::Asset,
                is_view_only: false,
                offered_actions: vec![GemEmptyStateAction::Swap],
            })
            .actions,
            vec![GemEmptyStateAction::Swap]
        );
        assert!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::Asset,
                is_view_only: false,
                offered_actions: vec![],
            })
            .actions
            .is_empty()
        );
        assert!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::Nfts,
                is_view_only: false,
                offered_actions: vec![],
            })
            .description
            .is_none(),
            "an nft list without a receive action says only that it is empty"
        );
        assert_eq!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::Nfts,
                is_view_only: false,
                offered_actions: vec![GemEmptyStateAction::Receive],
            })
            .description,
            Some(GemEmptyStateText::NftsDescription)
        );
    }

    #[test]
    fn test_a_search_without_a_custom_token_action_falls_back_to_the_plain_description() {
        assert_eq!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::SearchAssets,
                is_view_only: false,
                offered_actions: vec![],
            })
            .description,
            Some(GemEmptyStateText::SearchDescription)
        );
        assert_eq!(
            empty_state(GemEmptyStateInput {
                kind: GemEmptyStateKind::SearchAssets,
                is_view_only: false,
                offered_actions: vec![GemEmptyStateAction::AddCustomToken],
            })
            .description,
            Some(GemEmptyStateText::SearchAssetsDescription)
        );
    }
}
