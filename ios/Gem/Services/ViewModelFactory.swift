// Copyright (c). Gem Wallet. All rights reserved.

import Support
import class Gemstone.GemDeveloperService
import class Gemstone.GemDeviceService
import class Gemstone.GemWalletPreferencesService
import protocol Gemstone.GemSupportServiceProtocol
import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemAmountService
import class Gemstone.GemApiClient
import class Gemstone.GemAssetConfigService
import class Gemstone.GemAssetDiscoveryService
import class Gemstone.GemAssetsService
import class Gemstone.GemAvatarService
import class Gemstone.GemBalanceService
import class Gemstone.GemChartService
import class Gemstone.GemRecentActivityService
import class Gemstone.GemConfirmTransferService
import class Gemstone.GemConfirmService
import class Gemstone.GemContactService
import class Gemstone.GemCurrencyService
import class Gemstone.GemManageContactService
import class Gemstone.GemDeeplinkService
import class Gemstone.GemExplorerService
import class Gemstone.GemFiatService
import class Gemstone.GemNameService
import protocol Gemstone.GemNotificationPermissions
import class Gemstone.GemNotificationsService
import class Gemstone.GemChainService
import class Gemstone.GemNftService
import class Gemstone.GemNotificationService
import class Gemstone.GemServiceStatus
import class Gemstone.GemAppUpdateService
import class Gemstone.GemNodeService
import class Gemstone.GemWalletService
import class Gemstone.GemCollectibleService
import class Gemstone.GemFiatQuoteService
import class Gemstone.GemPaymentService
import class Gemstone.GemPerpetualDetailsService
import class Gemstone.GemReceiveService
import class Gemstone.GemRecipientService
import class Gemstone.GemRewardsService
import class Gemstone.GemPerpetualService
import class Gemstone.GemPortfolioService
import class Gemstone.GemPreferencesService
import class Gemstone.GemPriceAlertService
import class Gemstone.GemPriceService
import class Gemstone.GemSearchService
import class Gemstone.GemSignMessageService
import class Gemstone.GemSimulationFormatter
import class Gemstone.GemStakeService
import class Gemstone.GemStreamSubscriptionService
import class Gemstone.GemSwapQuoteService
import class Gemstone.GemSwapService
import class Gemstone.GemTransactionDetailsService
import class Gemstone.GemTransactionStateService
import class Gemstone.GemTransferService
import class Gemstone.GemWalletHomeService
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import Assets
import InAppNotifications
import PriceAlerts
import Contacts
import FiatConnect
import Foundation
import GemstoneServices
import LockManager
import ManageWallets
import MarketInsight
import NFT
import Onboarding
import Preferences
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Recents
import Perpetuals
import Settings
import Stake
import Store
import Swap
import SwiftUI
import Transactions
import Transfer
import WalletConnector
import WalletConnectorService
import WalletTab
import class Gemstone.GemAssetDetailsService
import class Gemstone.GemAddAssetService
import class Gemstone.GemAssetSelectionService
import class Gemstone.GemBannerService
import class Gemstone.GemTransactionsService
import struct Gemstone.GemTransferData

public struct ViewModelFactory: Sendable {
    let apiClient: GemApiClient
    let assetConfig: GemAssetConfigService
    let assetDiscoveryService: GemAssetDiscoveryService
    let assetsService: GemAssetsService
    let avatarService: GemAvatarService
    let bannerService: GemBannerService
    let balanceService: GemBalanceService
    let confirmService: GemConfirmService
    let contactService: GemContactService
    let manageContactService: GemManageContactService
    let deeplinkService: GemDeeplinkService
    let explorerService: GemExplorerService
    let fiatService: GemFiatService
    let gatewayService: GatewayService
    let hyperliquidObserverService: any PerpetualObservable
    let nameService: GemNameService
    let nftService: GemNftService
    let nodeService: GemNodeService
    let paymentService: GemPaymentService
    let perpetualService: GemPerpetualService
    let portfolioService: GemPortfolioService
    let preferencesService: GemPreferencesService
    let priceAlertService: GemPriceAlertService
    let priceService: GemPriceService
    let rewardsService: GemRewardsService
    let searchService: GemSearchService
    let simulationFormatter: GemSimulationFormatter
    let stakeService: GemStakeService
    let streamSubscriptionService: GemStreamSubscriptionService
    let swapService: GemSwapService
    let transactionStateService: GemTransactionStateService
    let transactionsService: GemTransactionsService
    let transferService: GemTransferService
    let walletService: GemWalletService
    let walletSessionService: GemWalletSessionService
    let serviceStatusService: GemServiceStatus
    let appUpdateService: GemAppUpdateService
    let inAppNotificationService: GemNotificationService

    let biometryService: any BiometryAuthenticatable
    let keystore: any Keystore
    let observablePreferences: ObservablePreferences
    let recentAssetsService: GemRecentActivityService
    let amountService: GemAmountService
    let toastPresenter: ToastPresenter
    let walletPreferencesService: GemWalletPreferencesService
    let signMessageService: GemSignMessageService
    let developerService: GemDeveloperService
    let deviceService: GemDeviceService
    let notificationPermissions: any GemNotificationPermissions
    let storeManager: StoreManager
    let supportService: any GemSupportServiceProtocol
    let supportTyping: ObservableSupportTyping


    @MainActor
    public func assetScene(
        wallet: Wallet,
        asset: Asset,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
    ) -> AssetSceneViewModel {
        AssetSceneViewModel(
            service: Gemstone.GemAssetDetailsService(
                assets: assetsService,
                balances: balanceService,
                transactions: transactionsService,
                banners: bannerService,
                swap: swapService,
                explorer: explorerService,
                priceAlerts: priceAlertService,
                stream: streamSubscriptionService,
                deeplinks: deeplinkService,
                session: walletSessionService,
            ),
            preferences: observablePreferences,
            input: AssetSceneInput(wallet: wallet, asset: asset),
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
        )
    }

    @MainActor
    public func portfolioScene(wallet: Wallet, defaultType: PortfolioType) -> PortfolioSceneViewModel {
        PortfolioSceneViewModel(wallet: wallet, service: portfolioService, preferences: observablePreferences, defaultType: defaultType)
    }

    @MainActor
    public func walletScene(
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>,
        isPresentingWallets: Binding<Bool>,
    ) -> WalletSceneViewModel {
        WalletSceneViewModel(
            service: walletHomeService(),
            observablePreferences: observablePreferences,
            collectionsModel: CollectionsViewModel(service: nftService, wallet: wallet),
            wallet: wallet,
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
            isPresentingWallets: isPresentingWallets,
        )
    }

    @MainActor
    public func notificationsScene() -> NotificationsViewModel {
        NotificationsViewModel(service: GemNotificationsService(device: deviceService, preferences: preferencesService, permissions: notificationPermissions))
    }

    @MainActor
    public func settingsScene(walletId: WalletId) -> SettingsViewModel {
        SettingsViewModel(walletId: walletId, service: walletSessionService, observablePreferences: observablePreferences)
    }

    @MainActor
    public func preferencesScene(currencyModel: CurrencySceneViewModel) -> PreferencesViewModel {
        PreferencesViewModel(currencyModel: currencyModel, service: preferencesService, preferences: observablePreferences)
    }

    @MainActor
    public func aboutUsScene() -> AboutUsViewModel {
        AboutUsViewModel(preferences: observablePreferences, service: appUpdateService)
    }

    @MainActor
    public func chainListSettingsScene() -> ChainListSettingsViewModel {
        ChainListSettingsViewModel(service: GemChainService())
    }

    @MainActor
    public func serviceStatusScene() -> ServiceStatusViewModel {
        ServiceStatusViewModel(service: serviceStatusService)
    }

    @MainActor
    public func priceAlertsScene() -> PriceAlertsSceneViewModel {
        PriceAlertsSceneViewModel(service: priceAlertService)
    }

    @MainActor
    public func assetPriceAlertsScene(walletId: WalletId, asset: Asset) -> AssetPriceAlertsViewModel {
        AssetPriceAlertsViewModel(service: priceAlertService, walletId: walletId, asset: asset)
    }

    @MainActor
    public func setPriceAlertScene(walletId: WalletId, asset: Asset, price: Double? = nil, onComplete: StringAction) -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(walletId: walletId, asset: asset, service: priceAlertService, price: price, onComplete: onComplete)
    }

    @MainActor
    public func inAppNotificationsScene() -> InAppNotificationsViewModel? {
        walletSessionService.currentWallet.map { InAppNotificationsViewModel(wallet: $0, service: inAppNotificationService) }
    }

    @MainActor
    public func perpetualsScene(
        wallet: Wallet,
        onSelectAssetType: @escaping (SelectAssetType) -> Void,
        onSelectAsset: @escaping (Asset) -> Void,
        onSelectPortfolio: @escaping () -> Void,
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            service: perpetualService,
            observerService: hyperliquidObserverService,
            recentAssetsService: recentAssetsService,
            onSelectAssetType: onSelectAssetType,
            onSelectAsset: onSelectAsset,
            onSelectPortfolio: onSelectPortfolio,
        )
    }

    @MainActor
    public func collectionsScene(wallet: Wallet) -> CollectionsViewModel {
        CollectionsViewModel(service: nftService, wallet: wallet)
    }

    @MainActor
    public func validatorSelectScene(
        type: ValidatorSelectType,
        chain: Chain,
        currentValidator: DelegationValidator?,
        validators: [DelegationValidator],
        selectValidator: @escaping (DelegationValidator) -> Void,
    ) -> ValidatorSelectSceneViewModel {
        ValidatorSelectSceneViewModel(
            service: stakeService,
            type: type,
            chain: chain,
            currentValidator: currentValidator,
            validators: validators,
            selectValidator: selectValidator,
        )
    }

    @MainActor
    public func currencyScene() -> CurrencySceneViewModel {
        CurrencySceneViewModel(
            currencyStorage: observablePreferences,
            service: GemCurrencyService(preferences: preferencesService, prices: priceService, device: deviceService),
        )
    }

    @MainActor
    public func walletSearchScene(
        wallet: Wallet,
        onDismissSearch: VoidAction,
        onSelectAssetAction: AssetAction,
        onAddToken: VoidAction,
    ) -> WalletSearchSceneViewModel {
        WalletSearchSceneViewModel(
            wallet: wallet,
            service: assetSelectionService(),
            recentModel: RecentAssetsModel(walletId: wallet.id, types: RecentActivityType.allCases, service: recentAssetsService),
            onDismissSearch: onDismissSearch,
            onSelectAssetAction: onSelectAssetAction,
            onAddToken: onAddToken,
        )
    }

    @MainActor
    public func networkAssetsScene(wallet: Wallet, chain: Chain, onManageAssets: @escaping () -> Void) -> NetworkAssetsSceneViewModel {
        NetworkAssetsSceneViewModel(wallet: wallet, chain: chain, service: walletHomeService(), onManageAssets: onManageAssets)
    }

    private func walletHomeService() -> GemWalletHomeService {
        GemWalletHomeService(
            balances: balanceService,
            discovery: assetDiscoveryService,
            banners: bannerService,
            walletPreferences: walletPreferencesService,
            preferences: preferencesService,
            session: walletSessionService,
        )
    }

    private func assetSelectionService() -> GemAssetSelectionService {
        GemAssetSelectionService(
            search: searchService,
            balances: balanceService,
            priceAlerts: priceAlertService,
            recentActivity: recentAssetsService,
            preferences: preferencesService,
            perpetuals: perpetualService,
            session: walletSessionService,
        )
    }

    @MainActor
    public func chartScene(
        asset: Asset,
        walletId: WalletId,
        onSetPriceAlert: @escaping (Asset) -> Void,
    ) -> ChartSceneViewModel {
        ChartSceneViewModel(
            service: Gemstone.GemChartService(
                api: apiClient,
                price: priceService,
                preferences: preferencesService,
                priceAlerts: priceAlertService,
                explorer: explorerService,
            ),
            assetModel: AssetViewModel(asset: asset),
            walletId: walletId,
            onSetPriceAlert: onSetPriceAlert,
        )
    }

    @MainActor
    public func transactionScene(
        transaction: TransactionExtended,
        walletId: WalletId,
        onHeaderAction: @escaping (TransactionHeaderAction) -> Void,
        onAddContact: @escaping (AddContactType) -> Void,
    ) -> TransactionSceneViewModel {
        TransactionSceneViewModel(
            transaction: transaction,
            walletId: walletId,
            service: Gemstone.GemTransactionDetailsService(explorer: explorerService, preferences: preferencesService),
            onHeaderAction: onHeaderAction,
            onAddContact: onAddContact,
        )
    }

    @MainActor
    public func transactionsScene(wallet: Wallet, type: TransactionsRequestType) -> TransactionsViewModel {
        TransactionsViewModel(service: transactionsService, wallet: wallet, type: type)
    }

    @MainActor
    public func supportChatScene() -> SupportChatSceneViewModel {
        SupportChatSceneViewModel(service: supportService, typing: supportTyping)
    }

    @MainActor
    public func developerScene(walletId: WalletId) -> DeveloperViewModel {
        DeveloperViewModel(
            walletId: walletId,
            service: developerService,
            transactionStore: storeManager.transactionStore,
            assetStore: storeManager.assetStore,
            stakeStore: storeManager.stakeStore,
            bannerStore: storeManager.bannerStore,
            priceStore: storeManager.priceStore,
        )
    }

    @MainActor
    public func lockScene() -> LockSceneViewModel {
        LockSceneViewModel(service: biometryService)
    }

    @MainActor
    public func securityScene() -> SecurityViewModel {
        SecurityViewModel(service: biometryService, preferences: observablePreferences)
    }

    @MainActor
    public func walletsScene(
        navigationPath: Binding<NavigationPath>,
        isPresentingCreateWalletSheet: Binding<Bool>,
        isPresentingImportWalletSheet: Binding<Bool>,
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            preferences: observablePreferences,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet,
        )
    }

    @MainActor
    public func walletDetailScene(navigationPath: Binding<NavigationPath>, wallet: Wallet) -> WalletDetailViewModel {
        WalletDetailViewModel(
            navigationPath: navigationPath,
            wallet: wallet,
            service: walletService,
            preferences: observablePreferences,
        )
    }

    @MainActor
    public func walletImageScene(wallet: Wallet) -> WalletImageViewModel {
        WalletImageViewModel(wallet: wallet, avatarService: avatarService)
    }

    @MainActor
    public func createWalletScene(onComplete: VoidAction) -> CreateWalletModel {
        CreateWalletModel(service: walletService, preferences: observablePreferences, avatarService: avatarService, onComplete: onComplete)
    }

    @MainActor
    public func importWalletScene(onComplete: VoidAction) -> ImportWalletViewModel {
        ImportWalletViewModel(
            service: walletService,
            preferences: observablePreferences,
            nameService: nameService,
            avatarService: avatarService,
            onComplete: onComplete,
        )
    }

    @MainActor
    public func contactsScene(mode: ContactsViewModel.Mode = .list) -> ContactsViewModel {
        ContactsViewModel(service: contactService, manageContact: manageContactScene, mode: mode)
    }

    @MainActor
    public func manageContactScene(mode: ManageContactViewModel.Mode) -> ManageContactViewModel {
        ManageContactViewModel(service: manageContactService, nameService: nameService, mode: mode)
    }

    @MainActor
    public func addAssetScene(wallet: Wallet) -> AddAssetSceneViewModel {
        AddAssetSceneViewModel(
            wallet: wallet,
            service: Gemstone.GemAddAssetService(assets: assetsService, balances: balanceService, explorer: explorerService),
        )
    }

    @MainActor
    public func selectAssetScene(selectType: SelectAssetType, selectAssetAction: AssetAction = .none) -> SelectAssetViewModel? {
        walletSessionService.currentWallet.map { selectAssetScene(wallet: $0, selectType: selectType, selectAssetAction: selectAssetAction) }
    }

    @MainActor
    public func selectAssetScene(
        wallet: Wallet,
        selectType: SelectAssetType,
        selectAssetAction: AssetAction = .none,
        chains: [Chain] = [],
    ) -> SelectAssetViewModel {
        SelectAssetViewModel(
            wallet: wallet,
            selectType: selectType,
            service: assetSelectionService(),
            recentAssetsService: recentAssetsService,
            selectAssetAction: selectAssetAction,
            chains: chains,
        )
    }

    @MainActor
    public func assetsResultsScene(
        wallet: Wallet,
        request: WalletSearchRequest,
        title: String,
        onSelectAsset: @escaping (Asset) -> Void,
    ) -> AssetsResultsSceneViewModel {
        AssetsResultsSceneViewModel(
            wallet: wallet,
            service: assetSelectionService(),
            request: request,
            title: title,
            onSelectAsset: onSelectAsset,
        )
    }

    @MainActor
    public func confirmTransferScene(
        wallet: Wallet,
        data: GemTransferData,
        confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
        simulation: SimulationResult? = nil,
        onComplete: VoidAction,
    ) -> ConfirmTransferSceneViewModel {
        ConfirmTransferSceneViewModel(
            request: ConfirmTransferRequest(
                wallet: wallet,
                data: data,
                simulation: simulation,
                delegate: confirmTransferDelegate,
            ),
            service: confirmTransferService(),
            onComplete: { [toastPresenter] in
                Task { await toastPresenter.present(.transfer(for: data.inputType)) }
                onComplete?()
            },
        )
    }

    private func confirmTransferService() -> GemConfirmTransferService {
        GemConfirmTransferService(
            confirm: confirmService,
            explorer: explorerService,
            names: nameService,
            assetConfig: assetConfig,
            signer: KeystoreTransactionSigner(keystore: keystore),
            password: GemstoneKeystorePassword(keystore: keystore),
            recentActivity: recentAssetsService,
            preferences: preferencesService,
        )
    }

    @MainActor
    public func perpetualScene(
        asset: Asset,
        wallet: Wallet,
        onTransferData: TransferDataAction,
        onPerpetualRecipientData: ((PerpetualRecipientData) -> Void)?,
    ) -> PerpetualSceneViewModel {
        PerpetualSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: GemPerpetualDetailsService(perpetuals: perpetualService, transactions: transactionsService, preferences: preferencesService, session: walletSessionService),
            observerService: hyperliquidObserverService,
            onTransferData: onTransferData,
            onPerpetualRecipientData: onPerpetualRecipientData,
        )
    }

    @MainActor
    public func collectibleScene(wallet: Wallet, assetData: NFTAssetData, isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService(nfts: nftService, avatars: avatarService, explorer: explorerService),
            isPresentingSelectedAssetInput: isPresentingSelectedAssetInput,
        )
    }

    @MainActor
    public func rewardsScene(activateCode: String?) -> RewardsViewModel? {
        try? RewardsViewModel(service: rewardsService, activateCode: activateCode)
    }

    @MainActor
    public func chainSettingsScene(chain: Chain) -> ChainSettingsSceneViewModel {
        ChainSettingsSceneViewModel(chain: chain, service: gatewayService.chainSettingsService(nodes: nodeService, explorer: explorerService))
    }

    @MainActor
    public func receiveScene(assetData: AssetData, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetData: assetData, wallet: wallet, service: receiveService())
    }

    @MainActor
    public func receiveScene(assetAddress: AssetAddress, wallet: Wallet) -> ReceiveViewModel {
        ReceiveViewModel(assetAddress: assetAddress, wallet: wallet, service: receiveService())
    }

    private func receiveService() -> GemReceiveService {
        GemReceiveService(balances: balanceService, assets: assetsService)
    }

    @MainActor
    public func recipientScene(
        wallet: Wallet,
        asset: Asset,
        type: RecipientAssetType,
        recipient: RecipientData? = .none,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) -> RecipientSceneViewModel {
        RecipientSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: GemRecipientService(names: nameService, payments: paymentService, session: walletSessionService),
            nameService: nameService,
            type: type,
            recipient: recipient,
            onRecipientDataAction: onRecipientDataAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func amountScene(
        input: AmountInput,
        wallet: Wallet,
        onTransferAction: TransferDataAction,
    ) -> AmountSceneViewModel {
        AmountSceneViewModel(
            input: input,
            wallet: wallet,
            service: amountService,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func fiatScene(
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            service: GemFiatQuoteService(fiat: fiatService, balances: balanceService, session: walletSessionService),
            assetAddress: assetAddress,
            wallet: wallet,
            type: type,
            amount: amount,
        )
    }

    @MainActor
    public func swapScene(
        input: SwapInput,
        onSwap: @escaping (GemTransferData) -> Void,
    ) -> SwapSceneViewModel {
        SwapSceneViewModel(
            service: GemSwapQuoteService(
                swap: swapService,
                preferences: preferencesService,
                balances: balanceService,
                stream: streamSubscriptionService,
                session: walletSessionService,
            ),
            input: input,
            onSwap: onSwap,
        )
    }

    @MainActor
    public func stakeScene(
        wallet: Wallet,
        chain: Chain,
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: StakeChain(rawValue: chain.rawValue)!, // Expected Only StakeChain accepted.
            service: stakeService,
        )
    }

    @MainActor
    public func earnScene(
        wallet: Wallet,
        asset: Asset,
    ) -> EarnSceneViewModel {
        EarnSceneViewModel(
            wallet: wallet,
            asset: asset,
            service: stakeService,
        )
    }

    @MainActor
    public func delegationScene(
        wallet: Wallet,
        delegation: Delegation,
        asset: Asset,
        validators: [DelegationValidator],
        onAmountInputAction: AmountInputAction,
        onTransferAction: TransferDataAction,
    ) -> DelegationSceneViewModel {
        DelegationSceneViewModel(
            wallet: wallet,
            model: DelegationViewModel(service: stakeService, delegation: delegation, asset: asset, formatter: .auto, currencyCode: stakeService.currency()),
            asset: asset,
            service: stakeService,
            validators: validators,
            onAmountInputAction: onAmountInputAction,
            onTransferAction: onTransferAction,
        )
    }

    @MainActor
    public func signMessageScene(
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) -> SignMessageSceneViewModel {
        SignMessageSceneViewModel(
            service: signMessageService,
            payload: payload,
            confirmTransferDelegate: confirmTransferDelegate,
        )
    }
}
