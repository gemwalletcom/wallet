// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import ConnectionStatusService
import Foundation
import Gemstone
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemFiatServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Localization
import GemstoneServices
import NativeProviderService
import Preferences
import Primitives
import PrimitivesComponents
import Store
import StreamService
import SwiftHTTPClient
import Transfer
import WalletConnector
import WalletConnectorService
import WebSocketClient

struct ServicesFactory {
    func makeServices(storages: AppResolver.Storages, navigation: NavigationStateManager) -> AppResolver.Services {
        let storeManager = storages.storeManager
        let securePreferences = SecurePreferences()
        let preferencesStore = GemstonePreferencesStore.application()
        let preferencesService = Gemstone.GemPreferencesService(store: preferencesStore)
        let observablePreferences = ObservablePreferences(preferencesService: preferencesService)
        let nodeService = GemNodeService(store: GemstoneNodeStore(store: storeManager.nodeStore), preferences: preferencesStore)
        let nativeProvider = NativeProvider(nodeProvider: nodeService)
        let deviceKeyService = Gemstone.GemDeviceKeyService(store: GemstoneSecurePreferencesStore(namespace: "gateway"))
        let deviceRegistrationClient = Self.makeDeviceApiClient(provider: nativeProvider, deviceKey: deviceKeyService)

        let gemstoneWalletStore = GemstoneWalletStore(store: storeManager.walletStore)
        let walletPreferencesService = Gemstone.GemWalletPreferencesService(store: GemstoneWalletPreferencesStore())
        let devicePlatform = MainActor.assumeIsolated { GemstoneDevicePlatform(preferencesService: preferencesService, deviceKeyService: deviceKeyService, securePreferences: securePreferences) }
        let deviceService = Gemstone.GemDeviceService(
            api: deviceRegistrationClient,
            subscriptions: Gemstone.GemSubscriptionService(api: deviceRegistrationClient, store: gemstoneWalletStore),
            walletStore: gemstoneWalletStore,
            platform: devicePlatform,
            preferences: preferencesService,
        )
        let deviceApiClient = Self.makeDeviceApiClient(provider: nativeProvider, deviceKey: deviceKeyService)
        deviceApiClient.setDeviceSyncPreflight(device: deviceService)

        let nodeProvider: any NodeURLProvidable = nodeService
        let connectionService = Gemstone.GemConnectionService()
        let connectionStatusObserver = ConnectionStatusObserver(
            connectionService: connectionService,
            monitors: [
                InternetConnectionMonitor(connectionService: connectionService),
            ],
        )
        let apiClient = Gemstone.GemApiClient(provider: nativeProvider)
        let staticApiClient = Gemstone.GemStaticApiClient(provider: nativeProvider)
        let priceService = Gemstone.GemPriceService(
            api: apiClient,
            store: GemstonePriceStore(priceStore: storeManager.priceStore, fiatRateStore: storeManager.fiatRateStore),
        )
        let gemstoneAssetStore = GemstoneAssetStore(assetStore: storeManager.assetStore, balanceStore: storeManager.balanceStore)
        let gemstoneFileStore = GemstoneFileStore()
        let gemstoneAddressStore = GemstoneAddressStore(store: storeManager.addressStore)
        let gemstoneBannerStore = GemstoneBannerStore(store: storeManager.bannerStore)
        let gemstoneNotificationStore = GemstoneNotificationStore(store: storeManager.inAppNotificationStore)
        let gatewayService = GatewayService(
            provider: nativeProvider,
            preferences: GemstonePreferencesStore(namespace: "gateway"),
            securePreferences: GemstoneSecurePreferencesStore(namespace: "gateway"),
        )
        let walletSessionService = Gemstone.GemWalletSessionService(store: GemstoneWalletSessionStore(store: preferencesStore), wallets: gemstoneWalletStore)
        let assetsService = gatewayService.assetsService(api: apiClient, store: gemstoneAssetStore, price: priceService, preferences: preferencesService, session: walletSessionService)
        let scanConfiguration = URLSessionConfiguration.default
        scanConfiguration.timeoutIntervalForRequest = TimeInterval(Config().getScanConfig().timeoutSeconds)
        let scanService = Gemstone.GemScanService(
            api: Self.makeDeviceApiClient(
                provider: NativeProvider(session: URLSession(configuration: scanConfiguration)),
                deviceKey: deviceKeyService,
            ),
        )
        let paymentService = Gemstone.GemPaymentService(provider: nativeProvider)
        let transactionSimulationService = GemSimulationService(provider: nativeProvider)
        let serviceStatusConfiguration = URLSessionConfiguration.default
        serviceStatusConfiguration.timeoutIntervalForRequest = TimeInterval(serviceStatusTimeoutSeconds())
        let serviceStatusService = Gemstone.GemServiceStatus(
            provider: NativeProvider(session: URLSession(configuration: serviceStatusConfiguration)),
        )
        let recentAssetsService = GemRecentActivityService(store: GemstoneRecentActivityStore(store: storeManager.recentActivityStore), session: walletSessionService)
        let explorerService = Gemstone.GemExplorerService(preferences: preferencesService)
        let walletService = Gemstone.GemWalletService(
            keystore: storages.keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: storages.keystore),
            store: gemstoneWalletStore,
            session: walletSessionService,
            appPreferences: preferencesService,
            files: gemstoneFileStore,
            preferences: walletPreferencesService,
            explorer: explorerService,
            addresses: gemstoneAddressStore,
            localizer: GemstoneLocalizer(),
        )
        let avatarService = Gemstone.GemAvatarService(wallets: gemstoneWalletStore, files: gemstoneFileStore, provider: nativeProvider)
        let webSocket = Self.makeWebSocket(deviceKeyService: deviceKeyService, reconnection: connectionService)
        let gemstonePriceAlertStore = GemstonePriceAlertStore(store: storeManager.priceAlertStore)
        let gemstoneBalanceStore = GemstoneBalanceStore(store: storeManager.balanceStore)
        let streamSubscriptionService = Gemstone.GemStreamSubscriptionService(
            balances: gemstoneBalanceStore,
            alerts: gemstonePriceAlertStore,
            connection: GemstoneStreamConnection(webSocket: webSocket),
        )
        let balanceService = gatewayService.balanceService(
            walletStore: gemstoneWalletStore,
            assetStore: gemstoneAssetStore,
            store: gemstoneBalanceStore,
            assets: assetsService,
            price: priceService,
            stream: streamSubscriptionService,
            preferences: preferencesService,
        )
        let stakeService = gatewayService.stakeService(
            staticApi: staticApiClient,
            store: GemstoneStakeStore(store: storeManager.stakeStore),
            addressStore: gemstoneAddressStore,
            explorer: explorerService,
            preferences: preferencesService,
            session: walletSessionService,
        )
        let nftService = Gemstone.GemNftService(api: deviceApiClient, store: GemstoneNftStore(store: storeManager.nftStore), session: walletSessionService)
        let transactionStateStore = GemstoneTransactionStateStore(store: storeManager.transactionStore)
        let transactionStateService = gatewayService.transactionStateService(
            store: transactionStateStore,
            assets: assetsService,
            balance: balanceService,
            stake: stakeService,
            nft: nftService,
        )
        let transactionsService = Gemstone.GemTransactionsService(
            api: deviceApiClient,
            assets: assetsService,
            store: GemstoneTransactionStore(store: storeManager.transactionStore),
            addressStore: gemstoneAddressStore,
            walletPreferences: walletPreferencesService,
            preferences: preferencesService,
            session: walletSessionService,
            transactionStatus: GemstoneTransactionStatusService(service: transactionStateService),
        )

        let pushNotificationEnablerService = PushNotificationEnablerService(preferencesService: preferencesService)
        let notificationPermissions = GemstoneNotificationPermissions(service: pushNotificationEnablerService)
        let bannerService = Gemstone.GemBannerService(store: gemstoneBannerStore)
        let navigationPresenter = NavigationPresenter(assetsService: assetsService, nftService: nftService, recentActivity: recentAssetsService)
        let gemstonePerpetualStore = GemstonePerpetualStore(store: storeManager.perpetualStore, balanceStore: storeManager.balanceStore)
        let perpetualService = gatewayService.perpetualService(
            price: priceService,
            store: gemstonePerpetualStore,
            assetStore: gemstoneAssetStore,
            preferences: preferencesService,
            balance: balanceService,
            walletPreferences: walletPreferencesService,
            session: walletSessionService,
        )
        let portfolioService = Gemstone.GemPortfolioService(
            api: deviceApiClient,
            store: GemstonePortfolioStore(assetStore: storeManager.assetStore),
            price: priceService,
            perpetual: perpetualService,
            preferences: preferencesService,
        )
        let priceAlertService = Gemstone.GemPriceAlertService(
            api: deviceApiClient,
            preferences: preferencesService,
            store: gemstonePriceAlertStore,
            device: deviceService,
            permissions: notificationPermissions,
        )
        let fiatService = Gemstone.GemFiatService(
            api: deviceApiClient,
            assets: assetsService,
            store: GemstoneFiatStore(store: storeManager.fiatTransactionStore),
        )
        let gemstoneSupportStore = GemstoneSupportStore(store: storeManager.supportChatStore)
        let supportService = Gemstone.GemSupportService(api: deviceApiClient, store: gemstoneSupportStore, files: gemstoneFileStore, provider: nativeProvider)
        let streamService = Gemstone.GemStreamService(
            price: priceService,
            priceAlert: priceAlertService,
            balance: balanceService,
            transactions: transactionsService,
            nft: nftService,
            perpetual: perpetualService,
            fiat: fiatService,
            notifications: gemstoneNotificationStore,
            support: gemstoneSupportStore,
            walletStore: gemstoneWalletStore,
        )
        let streamObserverService = StreamObserverService(
            subscriptionService: streamSubscriptionService,
            service: streamService,
            preferencesService: preferencesService,
            webSocket: webSocket,
        )
        let swapper = GemSwapper(rpcProvider: NativeProvider(nodeProvider: nodeProvider))
        let swapService = Gemstone.GemSwapService(
            swapper: swapper,
            keystore: storages.keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: storages.keystore),
            store: GemstoneSwapStore(
                assetStore: storeManager.assetStore,
                transactionStore: storeManager.transactionStore,
                recentActivityStore: storeManager.recentActivityStore,
            ),
        )

        let chainService = Gemstone.GemChainService()
        let addressService = Gemstone.GemAddressService()
        let walletConnectorPresenter = WalletConnectorPresenter()
        let walletConnectorInteractor = WalletConnectorInteractor(presenter: walletConnectorPresenter)
        let walletConnector = Self.makeWalletConnector(
            connectionsStore: storeManager.connectionsStore,
            interactor: walletConnectorInteractor,
            transactionSimulationService: transactionSimulationService,
            walletSessionService: walletSessionService,
            chainService: chainService,
        )

        let assetDiscoveryService = Gemstone.GemAssetDiscoveryService(
            api: deviceApiClient,
            balance: balanceService,
            transactions: transactionsService,
            nft: nftService,
            walletStore: gemstoneWalletStore,
            preferences: walletPreferencesService,
        )

        let configService = Gemstone.GemConfigService(api: apiClient, preferences: preferencesService)
        let appUpdateService = Gemstone.GemAppUpdateService(config: configService, preferences: preferencesService)
        let rateService = RateService(preferencesService: preferencesService)

        let appStartService = Gemstone.GemAppStartService(
            config: configService,
            banners: bannerService,
            assets: assetsService,
            balance: balanceService,
            walletConfiguration: Gemstone.GemWalletConfigurationService(
                api: deviceApiClient,
                banners: gemstoneBannerStore,
                preferences: walletPreferencesService,
            ),
            wallet: walletService,
            device: deviceService,
        )

        let onStartService = OnstartService(
            appStartService: appStartService,
            preferencesService: preferencesService,
            keystore: storages.keystore,
            session: walletSessionService,
        )

        let hyperliquidWebSocket = WebSocketConnection(
            url: nodeService.webSocketNode(for: .hyperCore),
            reconnection: connectionService,
        )
        let hyperliquidObserverService = HyperliquidObserverService(
            webSocket: hyperliquidWebSocket,
            perpetualService: perpetualService,
            streamService: Gemstone.GemPerpetualStreamService(
                perpetual: perpetualService,
                connection: PerpetualStreamConnection(webSocket: hyperliquidWebSocket),
            ),
        )

        let nameService = Gemstone.GemNameService(api: deviceApiClient, store: gemstoneAddressStore)
        let rewardsService = Gemstone.GemRewardsService(
            api: deviceApiClient,
            auth: Gemstone.GemAuthService(
                api: deviceApiClient,
                keystore: storages.keystore.gemKeystore,
                password: GemstoneKeystorePassword(keystore: storages.keystore),
                deviceKey: deviceKeyService,
            ),
            balance: balanceService,
        )
        let toastPresenter = ToastPresenter()
        let pushNotificationService = Gemstone.GemPushNotificationService()
        let navigationHandler = NavigationHandler(
            navigationState: navigation,
            presenter: navigationPresenter,
            assetsService: assetsService,
            assetStore: storeManager.assetStore,
            walletConnector: walletConnector,
            toastPresenter: toastPresenter,
            pushNotificationService: pushNotificationService,
            transactionStore: storeManager.transactionStore,
            deeplinkService: Gemstone.GemDeeplinkService(),
            paymentService: paymentService,
            transactionStateService: transactionStateService,
            walletConnectorPresenter: walletConnectorPresenter,
            walletSessionService: walletSessionService,
        )
        let searchService = Gemstone.GemSearchService(
            assets: assetsService,
            assetStore: gemstoneAssetStore,
            price: priceService,
            perpetualStore: gemstonePerpetualStore,
            store: GemstoneSearchStore(store: storeManager.searchStore, assetListStore: storeManager.assetListStore),
        )
        let inAppNotificationService = Gemstone.GemNotificationService(
            api: deviceApiClient,
            store: gemstoneNotificationStore,
            preferences: walletPreferencesService,
            session: walletSessionService,
        )

        let contactService = Gemstone.GemContactService(
            store: GemstoneContactStore(store: storeManager.contactStore),
            addressStore: gemstoneAddressStore,
            files: gemstoneFileStore,
        )

        let appLifecycleService = AppLifecycleService(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            deviceService: deviceService,
            subscriptionsObserver: storeManager.walletStore.observer(),
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualService: perpetualService,
            perpetualObserver: hyperliquidObserverService,
            walletSessionService: walletSessionService,
            transactionStateService: transactionStateService,
        )

        let confirmService = gatewayService.confirmService(
            simulation: transactionSimulationService,
            scanner: scanService,
            transactionState: transactionStateService,
            balance: balanceService,
            price: priceService,
            assets: assetsService,
            transactionStatus: GemstoneTransactionStatusService(service: transactionStateService),
        )
        let viewModelFactory = ViewModelFactory(
            apiClient: apiClient,
            assetConfig: Gemstone.GemAssetConfigService(),
            assetDiscoveryService: assetDiscoveryService,
            assetsService: assetsService,
            avatarService: avatarService,
            bannerService: bannerService,
            balanceService: balanceService,
            confirmService: confirmService,
            contactService: contactService,
            manageContactService: Gemstone.GemManageContactService(
                contacts: contactService,
                addresses: addressService,
                payments: paymentService,
            ),
            deeplinkService: Gemstone.GemDeeplinkService(),
            explorerService: explorerService,
            fiatService: fiatService,
            gatewayService: gatewayService,
            hyperliquidObserverService: hyperliquidObserverService,
            nameService: nameService,
            nftService: nftService,
            nodeService: nodeService,
            paymentService: paymentService,
            perpetualService: perpetualService,
            portfolioService: portfolioService,
            preferencesService: preferencesService,
            priceAlertService: priceAlertService,
            priceService: priceService,
            rewardsService: rewardsService,
            searchService: searchService,
            simulationFormatter: Gemstone.GemSimulationFormatter(),
            stakeService: stakeService,
            streamSubscriptionService: streamSubscriptionService,
            swapService: swapService,
            transactionStateService: transactionStateService,
            transactionsService: transactionsService,
            transferService: Gemstone.GemTransferService(),
            walletService: walletService,
            walletSessionService: walletSessionService,
            serviceStatusService: serviceStatusService,
            appUpdateService: appUpdateService,
            inAppNotificationService: inAppNotificationService,
            biometryService: BiometryAuthenticationService(
                keystorePassword: LocalKeystorePassword(),
                securityService: Gemstone.GemSecurityService(),
            ),
            keystore: storages.keystore,
            observablePreferences: observablePreferences,
            recentAssetsService: recentAssetsService,
            amountService: Gemstone.GemAmountService(stake: stakeService, preferences: preferencesService, session: walletSessionService),
            toastPresenter: toastPresenter,
            walletPreferencesService: walletPreferencesService,
            signMessageService: Gemstone.GemSignMessageService(
                names: nameService,
                explorer: explorerService,
                keystore: storages.keystore.gemKeystore,
                password: GemstoneKeystorePassword(keystore: storages.keystore),
            ),
            developerService: Gemstone.GemDeveloperService(
                platform: devicePlatform,
                preferences: preferencesService,
                walletPreferences: walletPreferencesService,
                transactions: transactionStateStore,
                perpetual: perpetualService,
            ),
            deviceService: deviceService,
            notificationPermissions: notificationPermissions,
            storeManager: storeManager,
            supportService: supportService,
            supportTyping: gemstoneSupportStore.typing,
        )

        return AppResolver.Services(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            deviceService: deviceService,
            navigationHandler: navigationHandler,
            navigationPresenter: navigationPresenter,
            streamObserverService: streamObserverService,
            transactionStateService: transactionStateService,
            observablePreferences: observablePreferences,
            walletSessionService: walletSessionService,
            appUpdateService: appUpdateService,
            rateService: rateService,
            onstartService: onStartService,
            appStartService: appStartService,
            pushNotificationEnablerService: pushNotificationEnablerService,
            walletConnectorPresenter: walletConnectorPresenter,
            toastPresenter: toastPresenter,
            viewModelFactory: viewModelFactory,
            appLifecycleService: appLifecycleService,
        )
    }
}

// MARK: - Private Static

extension ServicesFactory {
    private static func makeDeviceApiClient(
        provider: NativeProvider,
        deviceKey: Gemstone.GemDeviceKeyService,
    ) -> Gemstone.GemDeviceApiClient {
        Gemstone.GemDeviceApiClient(
            provider: provider,
            deviceKey: deviceKey,
        )
    }

    private static func makeWalletConnector(
        connectionsStore: ConnectionStore,
        interactor: WalletConnectorInteractor,
        transactionSimulationService: GemSimulationService,
        walletSessionService: GemWalletSessionService,
        chainService: Gemstone.GemChainService,
    ) -> WalletConnectorService {
        WalletConnectorService(
            walletSessionService: walletSessionService,
            interactor: interactor,
            service: GemWalletConnectService(
                simulation: transactionSimulationService,
                store: GemstoneConnectionStore(store: connectionsStore),
                signer: interactor,
                session: walletSessionService,
            ),
            chainService: chainService,
        )
    }

    private static func makeWebSocket(deviceKeyService: GemDeviceKeyService, reconnection: any Reconnectable) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(deviceKeyService: deviceKeyService)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider, reconnection: reconnection)
        return WebSocketConnection(configuration: configuration)
    }
}

final class GemstoneLocalizer: GemLocalizer, Sendable {
    func text(text: GemLocalizedText) -> String {
        switch text {
        case let .walletDefaultName(index):
            Localized.Wallet.defaultName(Int(index))
        case let .walletDefaultNameChain(chain, index):
            Localized.Wallet.defaultNameChain(Chain(core: chain).networkName, Int(index))
        }
    }
}
