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
import GemstoneServices
import Localization
import NativeProviderService
import Primitives
import PrimitivesComponents
import Store
import StreamService
import WalletConnector
import WalletConnectorService
import WebSocketClient

struct ServicesFactory {
    func makeServices(storages: AppResolver.Storages, navigation: NavigationStateManager) -> AppResolver.Services {
        let stores = storages.stores
        let preferencesStore = GemstonePreferencesStore.application()
        let preferencesService = Gemstone.GemPreferencesService(store: preferencesStore)
        let observablePreferences = ObservablePreferences(preferencesService: preferencesService)
        let nodeService = GemNodeService(store: GemstoneNodeStore(store: stores.nodeStore), preferences: preferencesStore)
        let nativeProvider = NativeProvider()
        let deviceKeyService = Gemstone.GemDeviceKeyService(store: GemstoneSecurePreferencesStore(namespace: "gateway"))
        let deviceRegistrationClient = Self.makeDeviceApiClient(provider: nativeProvider, deviceKey: deviceKeyService)

        let gemstoneWalletStore = GemstoneWalletStore(store: stores.walletStore)
        let walletSessionService = Gemstone.GemWalletSessionService(store: GemstoneWalletSessionStore(store: preferencesStore), wallets: gemstoneWalletStore)
        let walletPreferencesService = Gemstone.GemWalletPreferencesService(store: GemstoneWalletPreferencesStore())
        let devicePlatform = MainActor.assumeIsolated { GemstoneDevicePlatform(preferencesService: preferencesService, deviceKeyService: deviceKeyService) }
        let deviceService = Gemstone.GemDeviceService(
            api: deviceRegistrationClient,
            subscriptions: Gemstone.GemSubscriptionService(api: deviceRegistrationClient, session: walletSessionService),
            session: walletSessionService,
            platform: devicePlatform,
            preferences: preferencesService,
        )
        let deviceApiClient = Self.makeDeviceApiClient(provider: nativeProvider, deviceKey: deviceKeyService)
        deviceApiClient.setDeviceSyncPreflight(device: deviceService)

        let connectionService = Gemstone.GemConnectionService()
        let streamHealth = ConnectionComponentHealth(component: .stream)
        let connectionStatusObserver = ConnectionStatusObserver(
            connectionService: connectionService,
            monitors: [
                InternetConnectionMonitor(),
                streamHealth,
            ],
        )
        let apiClient = Gemstone.GemApiClient(provider: nativeProvider)
        let staticApiClient = Gemstone.GemStaticApiClient(provider: nativeProvider)
        let priceService = Gemstone.GemPriceService(
            store: GemstonePriceStore(priceStore: stores.priceStore),
            preferences: preferencesService,
        )
        let gemstoneAssetStore = GemstoneAssetStore(assetStore: stores.assetStore, balanceStore: stores.balanceStore)
        let gemstoneFileStore = GemstoneFileStore()
        let gemstoneAddressStore = GemstoneAddressStore(store: stores.addressStore)
        let gemstoneBannerStore = GemstoneBannerStore(store: stores.bannerStore)
        let gemstoneNotificationStore = GemstoneNotificationStore(store: stores.inAppNotificationStore)
        let gatewayService = GatewayService(
            provider: nativeProvider,
            nodes: nodeService,
            preferences: GemstonePreferencesStore(namespace: "gateway"),
            securePreferences: GemstoneSecurePreferencesStore(namespace: "gateway"),
        )
        let nameService = Gemstone.GemNameService(api: deviceApiClient, store: gemstoneAddressStore)
        let assetsService = gatewayService.assetsService(api: apiClient, store: gemstoneAssetStore, price: priceService, preferences: preferencesService, session: walletSessionService)
        let scanConfiguration = URLSessionConfiguration.default
        scanConfiguration.timeoutIntervalForRequest = GemConstants.scanTimeout.timeInterval
        let scanService = Gemstone.GemScanService(
            api: Self.makeDeviceApiClient(
                provider: NativeProvider(session: URLSession(configuration: scanConfiguration)),
                deviceKey: deviceKeyService,
            ),
        )
        let paymentService = Gemstone.GemPaymentService(provider: nativeProvider, assets: assetsService)
        let transactionSimulationService = GemSimulationService(provider: nativeProvider, nodes: nodeService)
        let webSocket = Self.makeWebSocket(deviceKeyService: deviceKeyService, reconnection: connectionService)
        let serviceStatusConfiguration = URLSessionConfiguration.default
        serviceStatusConfiguration.timeoutIntervalForRequest = GemConstants.serviceStatusTimeout.timeInterval
        let serviceStatusService = Gemstone.GemServiceStatus(
            provider: NativeProvider(session: URLSession(configuration: serviceStatusConfiguration)),
            stream: GemstoneStreamConnection(webSocket: webSocket),
        )
        let recentAssetsService = GemRecentActivityService(store: GemstoneRecentActivityStore(store: stores.recentActivityStore), session: walletSessionService)
        let explorerService = Gemstone.GemExplorerService(preferences: preferencesService)
        let avatarService = Gemstone.GemAvatarService(wallets: gemstoneWalletStore, files: gemstoneFileStore, provider: nativeProvider)
        let walletService = storages.keystore.walletService(
            store: gemstoneWalletStore,
            session: walletSessionService,
            appPreferences: preferencesService,
            files: gemstoneFileStore,
            preferences: walletPreferencesService,
            explorer: explorerService,
            names: nameService,
            avatar: avatarService,
        )
        let pushNotificationEnablerService = PushNotificationEnablerService(preferencesService: preferencesService)
        let notificationPermissions = GemstoneNotificationPermissions(service: pushNotificationEnablerService)
        let priceAlertService = Gemstone.GemPriceAlertService(
            api: deviceApiClient,
            preferences: preferencesService,
            store: GemstonePriceAlertStore(store: stores.priceAlertStore),
            permissions: notificationPermissions,
        )
        let gemstoneBalanceStore = GemstoneBalanceStore(store: stores.balanceStore)
        let streamSubscriptionService = Gemstone.GemStreamSubscriptionService(
            balances: gemstoneBalanceStore,
            alerts: priceAlertService,
            connection: GemstoneStreamConnection(webSocket: webSocket),
        )
        let balanceService = gatewayService.balanceService(
            store: gemstoneBalanceStore,
            assets: assetsService,
            session: walletSessionService,
            stream: streamSubscriptionService,
        )
        let stakeService = gatewayService.stakeService(
            staticApi: staticApiClient,
            store: GemstoneStakeStore(store: stores.stakeStore),
            names: nameService,
            explorer: explorerService,
            preferences: preferencesService,
            session: walletSessionService,
        )
        let nftService = Gemstone.GemNftService(api: deviceApiClient, store: GemstoneNftStore(store: stores.nftStore), session: walletSessionService)
        let transactionStateStore = GemstoneTransactionStateStore(store: stores.transactionStore)
        let transactionStateService = gatewayService.transactionStateService(
            store: transactionStateStore,
            assets: assetsService,
            balance: balanceService,
            stake: stakeService,
            nft: nftService,
            payments: paymentService,
        )
        transactionStateService.setStatus(status: GemstoneTransactionStatusService(service: transactionStateService))
        let transactionsService = Gemstone.GemTransactionsService(
            api: deviceApiClient,
            assets: assetsService,
            store: transactionStateStore,
            names: nameService,
            walletPreferences: walletPreferencesService,
            session: walletSessionService,
            transactionStatus: GemstoneTransactionStatusService(service: transactionStateService),
        )

        let bannerService = Gemstone.GemBannerService(store: gemstoneBannerStore, platform: .ios)
        let navigationPresenter = NavigationPresenter(assetsService: assetsService, nftService: nftService)
        let gemstonePerpetualStore = GemstonePerpetualStore(store: stores.perpetualStore)
        let perpetualService = gatewayService.perpetualService(
            price: priceService,
            store: gemstonePerpetualStore,
            assets: assetsService,
            preferences: preferencesService,
            balance: balanceService,
            walletPreferences: walletPreferencesService,
            session: walletSessionService,
            recentActivity: recentAssetsService,
        )
        let portfolioService = Gemstone.GemPortfolioService(
            api: deviceApiClient,
            store: GemstonePortfolioStore(assetStore: stores.assetStore),
            price: priceService,
            perpetual: perpetualService,
            preferences: preferencesService,
        )
        let fiatService = Gemstone.GemFiatService(
            api: deviceApiClient,
            assets: assetsService,
            store: GemstoneFiatStore(store: stores.fiatTransactionStore),
        )
        let gemstoneSupportStore = GemstoneSupportStore(store: stores.supportChatStore)
        let supportService = Gemstone.GemSupportService(api: deviceApiClient, store: gemstoneSupportStore, files: gemstoneFileStore, provider: nativeProvider)
        let inAppNotificationService = Gemstone.GemNotificationService(
            api: deviceApiClient,
            store: gemstoneNotificationStore,
            preferences: walletPreferencesService,
            session: walletSessionService,
        )
        let streamService = Gemstone.GemStreamService(
            price: priceService,
            priceAlert: priceAlertService,
            balance: balanceService,
            transactions: transactionsService,
            nft: nftService,
            perpetual: perpetualService,
            fiat: fiatService,
            notifications: inAppNotificationService,
            support: supportService,
            subscriptions: streamSubscriptionService,
            session: walletSessionService,
            device: deviceService,
        )
        let streamObserverService = StreamObserverService(
            service: streamService,
            webSocket: webSocket,
            health: streamHealth,
            reconnection: connectionService,
        )
        let swapper = GemSwapper(rpcProvider: NativeProvider(), nodes: nodeService)
        let swapService = storages.keystore.swapService(
            swapper: swapper,
            store: GemstoneSwapStore(
                assetStore: stores.assetStore,
                transactionStore: stores.transactionStore,
                recentActivityStore: stores.recentActivityStore,
            ),
        )

        let chainService = Gemstone.GemChainService()
        let addressService = Gemstone.GemAddressService()
        let signMessageService = storages.keystore.signMessageService(
            names: nameService,
            explorer: explorerService,
        )
        let walletConnectorPresenter = WalletConnectorPresenter()
        let walletConnectorInteractor = WalletConnectorInteractor(presenter: walletConnectorPresenter)
        let walletConnectService = Gemstone.GemWalletConnectService(
            simulation: transactionSimulationService,
            scanner: scanService,
            store: GemstoneConnectionStore(store: stores.connectionsStore),
            signer: walletConnectorInteractor,
            session: walletSessionService,
            assets: assetsService,
            signMessage: signMessageService,
        )
        let walletConnector = WalletConnectorService(
            walletSessionService: walletSessionService,
            interactor: walletConnectorInteractor,
            service: walletConnectService,
            chainService: chainService,
        )

        let assetDiscoveryService = Gemstone.GemAssetDiscoveryService(
            api: deviceApiClient,
            balance: balanceService,
            transactions: transactionsService,
            nft: nftService,
            session: walletSessionService,
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
                banners: bannerService,
                preferences: walletPreferencesService,
            ),
            wallet: walletService,
            device: deviceService,
            support: supportService,
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

        let rewardsService = Gemstone.GemRewardsService(
            api: deviceApiClient,
            auth: storages.keystore.authService(
                api: deviceApiClient,
                deviceKey: deviceKeyService,
            ),
            balance: balanceService,
        )
        let toastPresenter = ToastPresenter()
        let pushNotificationService = Gemstone.GemPushNotificationService()
        let navigationRouter = NavigationRouter(
            navigationState: navigation,
            presenter: navigationPresenter,
            assetsService: assetsService,
            assetStore: stores.assetStore,
            walletConnector: walletConnector,
            toastPresenter: toastPresenter,
            pushNotificationService: pushNotificationService,
            transactionStore: stores.transactionStore,
            deeplinkService: Gemstone.GemDeeplinkService(),
            navigationService: Gemstone.GemNavigationService(assets: assetsService, session: walletSessionService, transactionState: transactionStateService),
            paymentService: paymentService,
            transactionStateService: transactionStateService,
            walletConnectorPresenter: walletConnectorPresenter,
            walletSessionService: walletSessionService,
        )
        let searchService = Gemstone.GemSearchService(
            assets: assetsService,
            price: priceService,
            perpetuals: perpetualService,
            store: GemstoneSearchStore(store: stores.searchStore, assetListStore: stores.assetListStore),
        )

        let contactService = Gemstone.GemContactService(
            store: GemstoneContactStore(store: stores.contactStore),
            names: nameService,
            files: gemstoneFileStore,
        )

        let appLifecycleService = AppLifecycleService(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            deviceService: deviceService,
            subscriptionsObserver: stores.walletStore.observer(),
            streamObserverService: streamObserverService,
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
            assetDiscoveryService: assetDiscoveryService,
            assetsService: assetsService,
            avatarService: avatarService,
            bannerService: bannerService,
            balanceService: balanceService,
            confirmService: confirmService,
            contactService: contactService,
            contactEditorService: Gemstone.GemContactEditorService(
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
            transactionsService: transactionsService,
            walletService: walletService,
            walletSessionService: walletSessionService,
            walletConnectService: walletConnectService,
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
            signMessageService: signMessageService,
            devicePlatform: devicePlatform,
            developerService: Gemstone.GemDeveloperService(
                platform: devicePlatform,
                preferences: preferencesService,
                walletPreferences: walletPreferencesService,
                transactionState: transactionStateService,
                transactions: transactionsService,
                perpetual: perpetualService,
                store: GemstoneDeveloperStore(
                    transactionStore: stores.transactionStore,
                    assetStore: stores.assetStore,
                    stakeStore: stores.stakeStore,
                    bannerStore: stores.bannerStore,
                    priceStore: stores.priceStore,
                ),
            ),
            deviceService: deviceService,
            notificationPermissions: notificationPermissions,
            stores: stores,
            supportService: supportService,
            supportTyping: gemstoneSupportStore.typing,
        )

        return AppResolver.Services(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            devicePlatform: devicePlatform,
            deviceService: deviceService,
            navigationRouter: navigationRouter,
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

    private static func makeWebSocket(deviceKeyService: GemDeviceKeyService, reconnection: any Reconnectable) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(deviceKeyService: deviceKeyService)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider, reconnection: reconnection)
        return WebSocketConnection(configuration: configuration)
    }
}
