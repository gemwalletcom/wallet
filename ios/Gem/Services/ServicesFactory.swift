// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemFiatServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import AppService
import ConnectionsService
import ConnectionStatusService
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemAPI
import Gemstone
import GemstonePrimitives
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
        let storeManager = StoreManager(db: storages.db)
        let securePreferences = SecurePreferences()
        let preferencesService = Gemstone.GemPreferencesService(store: GemstonePreferencesStore(namespace: "gemstone_"))
        let nodeService = NodeService(
            nodeStore: storeManager.nodeStore,
            service: GemNodeService(store: GemstoneNodeStore(store: storeManager.nodeStore)),
        )
        let nativeProvider = NativeProvider(nodeProvider: nodeService)
        let devicePrivateKey = (try? DeviceService.getOrCreateKeyPair(securePreferences: securePreferences))?.privateKey ?? Data()
        let deviceRegistrationClient = Self.makeDeviceApiClient(provider: nativeProvider, devicePrivateKey: devicePrivateKey)

        let gemWalletStore = GemstoneWalletStore(store: storeManager.walletStore)
        let gemDeviceStore = GemstoneDeviceStore()
        let gemDeviceService = Gemstone.GemDeviceService(
            api: deviceRegistrationClient,
            subscriptions: Gemstone.GemSubscriptionService(api: deviceRegistrationClient, store: gemWalletStore),
            walletStore: gemWalletStore,
            store: gemDeviceStore,
        )
        let deviceService = DeviceService(
            deviceProvider: gemDeviceService,
            preferencesService: preferencesService,
            securePreferences: securePreferences,
        )
        let gemDeviceApiClient = Self.makeDeviceApiClient(
            provider: nativeProvider,
            devicePrivateKey: devicePrivateKey,
            preflight: DeviceSyncPreflight(deviceService: deviceService),
        )
        let deviceObserverService = Self.makeDeviceObserverService(
            deviceService: deviceService,
            walletStore: storeManager.walletStore,
        )

        let nodeProvider: any NodeURLFetchable = nodeService
        let connectionStatusObserver = ConnectionStatusObserver(
            monitors: [
                InternetConnectionMonitor(),
            ],
        )
        let gemApiClient = Gemstone.GemApiClient(provider: nativeProvider, baseUrl: Constants.apiURL.absoluteString)
        let gemStaticApiClient = Gemstone.GemStaticApiClient(provider: nativeProvider, baseUrl: Constants.assetsURL.absoluteString)
        let chartService = Gemstone.GemChartService(api: gemApiClient)
        let gemPriceService = Gemstone.GemPriceService(
            api: gemApiClient,
            store: GemstonePriceStore(priceStore: storeManager.priceStore, fiatRateStore: storeManager.fiatRateStore),
        )
        let marketService = gemPriceService
        let priceService = gemPriceService
        let gemAssetStore = GemstoneAssetStore(assetStore: storeManager.assetStore, balanceStore: storeManager.balanceStore)
        let gatewayService = GatewayService(
            provider: nativeProvider,
            preferences: GemstonePreferencesStore(namespace: "gateway"),
            securePreferences: GemstoneSecurePreferencesStore(namespace: "gateway"),
        )
        let gemAssetsService = gatewayService.assetsService(api: gemApiClient, store: gemAssetStore, price: gemPriceService, preferences: preferencesService)
        let gemTransactionsService = Gemstone.GemTransactionsService(
            api: gemDeviceApiClient,
            assets: gemAssetsService,
            store: GemstoneTransactionStore(store: storeManager.transactionStore),
            addressStore: GemstoneAddressStore(store: storeManager.addressStore),
        )
        let gemScanService = Gemstone.GemScanService(api: gemDeviceApiClient)
        let paymentService = PaymentService(provider: nativeProvider)
        let transactionSimulationService = TransactionSimulationService(provider: nativeProvider)
        let serviceStatusConfiguration = URLSessionConfiguration.default
        serviceStatusConfiguration.timeoutIntervalForRequest = TimeInterval(serviceStatusTimeoutSeconds())
        let serviceStatusService = Gemstone.GemServiceStatus(
            provider: NativeProvider(session: URLSession(configuration: serviceStatusConfiguration), url: Constants.apiURL),
        )
        let chainServiceFactory = ChainServiceFactory(gatewayService: gatewayService)
        let gemWalletSessionService = Gemstone.GemWalletSessionService(store: GemstoneWalletSessionStore(preferences: storages.observablePreferences), wallets: gemWalletStore)
        let walletSessionService = WalletSessionService(service: gemWalletSessionService)
        let gemWalletService = Gemstone.GemWalletService(
            keystore: storages.keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: storages.keystore, walletStore: storeManager.walletStore),
            store: gemWalletStore,
            session: gemWalletSessionService,
            deviceStore: gemDeviceStore,
        )
        let avatarService = Gemstone.GemAvatarService(wallets: gemWalletStore, files: GemstoneFileStore(), provider: nativeProvider)
        let walletService = WalletService(
            service: gemWalletService,
            keystore: storages.keystore,
            walletSessionService: walletSessionService,
            preferences: storages.observablePreferences,
            avatarService: avatarService,
        )
        let gemBalanceService = gatewayService.balanceService(
            walletStore: gemWalletStore,
            assetStore: gemAssetStore,
            store: GemstoneBalanceStore(store: storeManager.balanceStore),
            assets: gemAssetsService,
            price: gemPriceService,
        )
        let balanceService = gemBalanceService
        let gemStakeService = gatewayService.stakeService(
            staticApi: gemStaticApiClient,
            store: GemstoneStakeStore(store: storeManager.stakeStore, addressStore: storeManager.addressStore),
        )
        let stakeService = gemStakeService
        let gemNftService = Gemstone.GemNftService(api: gemDeviceApiClient, store: GemstoneNftStore(store: storeManager.nftStore))
        let nftService = gemNftService
        let transactionsService = gemTransactionsService
        let transactionStateScheduler = Self.makeTransactionService(
            transactionStore: storeManager.transactionStore,
            gatewayService: gatewayService,
            stakeService: stakeService,
            nftService: nftService,
            balanceService: balanceService,
        )

        let preferences = storages.observablePreferences.preferences
        let pushNotificationEnablerService = PushNotificationEnablerService(preferences: preferences)
        let bannerService = Gemstone.GemBannerService(
            store: GemstoneBannerStore(store: storeManager.bannerStore),
            permissions: GemstoneNotificationPermissions(service: pushNotificationEnablerService),
        )
        let navigationPresenter = NavigationPresenter()
        let portfolioService = Gemstone.GemPortfolioService(api: gemDeviceApiClient, store: GemstonePortfolioStore(assetStore: storeManager.assetStore))
        let gemPerpetualStore = GemstonePerpetualStore(store: storeManager.perpetualStore, assetStore: storeManager.assetStore, balanceStore: storeManager.balanceStore)
        let gemPerpetualService = gatewayService.perpetualService(price: gemPriceService, store: gemPerpetualStore, preferences: preferencesService)
        let perpetualService = PerpetualService(
            provider: PerpetualProviderFactory(gatewayService: gatewayService, nodeProvider: nodeProvider).createProvider(),
            service: gemPerpetualService,
        )
        let webSocket = Self.makeWebSocket(securePreferences: securePreferences)
        let streamSubscriptionService = Gemstone.GemStreamSubscriptionService(
            price: gemPriceService,
            connection: GemstoneStreamConnection(webSocket: webSocket),
        )
        let priceAlertService = Gemstone.GemPriceAlertService(
            api: gemDeviceApiClient,
            preferences: preferencesService,
            store: GemstonePriceAlertStore(store: storeManager.priceAlertStore),
            device: GemstoneDeviceSync(service: deviceService),
            permissions: GemstoneNotificationPermissions(service: pushNotificationEnablerService),
        )
        let gemFiatService = Gemstone.GemFiatService(
            api: gemDeviceApiClient,
            assets: gemAssetsService,
            store: GemstoneFiatStore(store: storeManager.fiatTransactionStore),
        )
        let fiatService = gemFiatService
        let supportTypingState = SupportTypingState()
        let gemSupportStore = GemstoneSupportStore(store: storeManager.supportChatStore, typing: supportTypingState)
        let supportChatService = SupportChatService(
            provider: Gemstone.GemSupportService(api: gemDeviceApiClient, store: gemSupportStore),
            typing: supportTypingState,
        )
        let streamService = Gemstone.GemStreamService(
            price: gemPriceService,
            priceAlert: priceAlertService,
            balance: gemBalanceService,
            transactions: gemTransactionsService,
            nft: gemNftService,
            perpetual: gemPerpetualService,
            fiat: gemFiatService,
            notifications: GemstoneNotificationStore(store: storeManager.inAppNotificationStore),
            support: gemSupportStore,
            walletStore: gemWalletStore,
        )
        let streamObserverService = StreamObserverService(
            subscriptionService: streamSubscriptionService,
            service: streamService,
            preferences: preferences,
            webSocket: webSocket,
        )
        let explorerService = Gemstone.GemExplorerService(preferences: preferencesService)
        let gemSwapper = GemSwapper(rpcProvider: NativeProvider(nodeProvider: nodeProvider))
        let swapService = Gemstone.GemSwapService(
            swapper: gemSwapper,
            keystore: storages.keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: storages.keystore, walletStore: storeManager.walletStore),
        )

        let presenter = WalletConnectorPresenter()
        let walletConnectorManager = WalletConnectorManager(presenter: presenter)
        let connectionsService = Self.makeConnectionsService(
            connectionsStore: storeManager.connectionsStore,
            walletSessionService: walletSessionService,
            interactor: walletConnectorManager,
            transactionSimulationService: transactionSimulationService,
        )

        let assetsEnabler = AssetsEnablerService(
            service: gemBalanceService,
            priceUpdater: streamSubscriptionService,
            preferences: preferences,
        )
        let assetDiscoveryService = Gemstone.GemAssetDiscoveryService(
            api: gemDeviceApiClient,
            balance: gemBalanceService,
            transactions: gemTransactionsService,
            nft: gemNftService,
            walletStore: gemWalletStore,
            store: GemstoneAssetDiscoveryStore(),
        )

        let gemConfigService = Gemstone.GemConfigService(api: gemApiClient, preferences: preferencesService)
        let appUpdateService = Gemstone.GemAppUpdateService(config: gemConfigService, preferences: preferencesService)
        let rateService = RateService(preferences: preferences)

        let onStartService = OnstartService(
            assetsService: gemAssetsService,
            nodeStore: storeManager.nodeStore,
            preferences: preferences,
            preferencesService: preferencesService,
            walletService: walletService,
        )
        let appStartService = Gemstone.GemAppStartService(
            config: gemConfigService,
            banners: bannerService,
            assets: gemAssetsService,
            walletConfiguration: Gemstone.GemWalletConfigurationService(
                api: gemDeviceApiClient,
                banners: GemstoneBannerStore(store: storeManager.bannerStore),
                store: GemstoneWalletConfigurationStore(),
            ),
        )

        let hyperliquidObserverService = HyperliquidObserverService(
            nodeProvider: PerpetualNodeService(nodeProvider: nodeProvider),
            perpetualService: perpetualService,
        )

        let gemNameService = Gemstone.GemNameService(api: gemDeviceApiClient, store: GemstoneAddressStore(store: storeManager.addressStore))
        let rewardsService = Gemstone.GemRewardsService(
            api: gemDeviceApiClient,
            auth: Gemstone.GemAuthService(
                api: gemDeviceApiClient,
                keystore: storages.keystore.gemKeystore,
                password: GemstoneKeystorePassword(keystore: storages.keystore, walletStore: storeManager.walletStore),
                devicePrivateKey: devicePrivateKey,
            ),
        )
        let toastPresenter = ToastPresenter()
        let navigationHandler = NavigationHandler(
            navigationState: navigation,
            presenter: navigationPresenter,
            assetsService: gemAssetsService,
            assetStore: storeManager.assetStore,
            connectionsService: connectionsService,
            toastPresenter: toastPresenter,
            paymentService: paymentService,
            transactionStore: storeManager.transactionStore,
            walletConnectorPresenter: presenter,
            walletSessionService: walletSessionService,
        )
        let searchService = Gemstone.GemSearchService(
            assets: gemAssetsService,
            assetStore: gemAssetStore,
            price: gemPriceService,
            perpetualStore: gemPerpetualStore,
            store: GemstoneSearchStore(store: storeManager.searchStore, assetListStore: storeManager.assetListStore),
        )
        let inAppNotificationService = Gemstone.GemNotificationService(
            api: gemDeviceApiClient,
            store: GemstoneNotificationStore(store: storeManager.inAppNotificationStore),
        )

        let contactService = Gemstone.GemContactService(
            store: GemstoneContactStore(store: storeManager.contactStore),
            addressStore: GemstoneAddressStore(store: storeManager.addressStore),
            files: GemstoneFileStore(),
        )

        let perpetualEnablerService = PerpetualEnablerService(
            observer: hyperliquidObserverService,
            service: perpetualService,
            preferences: preferences,
        )
        let appLifecycleService = AppLifecycleService(
            preferences: preferences,
            connectionsService: connectionsService,
            connectionStatusObserver: connectionStatusObserver,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualEnablerService: perpetualEnablerService,
            walletSessionService: walletSessionService,
        )

        let gemConfirmService = gatewayService.confirmService(
            simulation: transactionSimulationService,
            scanner: gemScanService,
        )
        let viewModelFactory = ViewModelFactory(
            keystore: storages.keystore,
            chainServiceFactory: chainServiceFactory,
            gemConfirmService: gemConfirmService,
            swapService: swapService,
            assetsEnabler: assetsEnabler,
            priceUpdater: streamSubscriptionService,
            walletSessionService: walletSessionService,
            stakeService: stakeService,
            explorerService: explorerService,
            amountService: AmountService(stakeService: stakeService),
            nameService: gemNameService,
            balanceService: balanceService,
            balanceStore: storeManager.balanceStore,
            addressStore: storeManager.addressStore,
            priceService: priceService,
            priceStore: storeManager.priceStore,
            transactionStateScheduler: transactionStateScheduler,
            gemNameService: gemNameService,
            recentActivityStore: storeManager.recentActivityStore,
            toastPresenter: toastPresenter,
            fiatService: fiatService,
            assetsService: gemAssetsService,
            assetStore: storeManager.assetStore,
            priceAlertService: priceAlertService,
            searchService: searchService,
            perpetualService: perpetualService,
        )

        return AppResolver.Services(
            balanceService: balanceService,
            bannerService: bannerService,
            chainServiceFactory: chainServiceFactory,
            connectionsService: connectionsService,
            connectionStatusObserver: connectionStatusObserver,
            deviceService: deviceService,
            nodeService: nodeService,
            serviceStatusService: serviceStatusService,
            navigationHandler: navigationHandler,
            navigationPresenter: navigationPresenter,
            priceAlertService: priceAlertService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            priceService: priceService,
            chartService: chartService,
            marketService: marketService,
            stakeService: stakeService,
            transactionsService: transactionsService,
            transactionStateScheduler: transactionStateScheduler,
            walletService: walletService,
            walletSessionService: walletSessionService,
            assetsEnabler: assetsEnabler,
            assetDiscoveryService: assetDiscoveryService,
            gemAssetsService: gemAssetsService,
            explorerService: explorerService,
            gatewayService: gatewayService,
            nftService: nftService,
            avatarService: avatarService,
            swapService: swapService,
            appUpdateService: appUpdateService,
            rateService: rateService,
            deviceObserverService: deviceObserverService,
            onstartService: onStartService,
            appStartService: appStartService,
            pushNotificationEnablerService: pushNotificationEnablerService,
            walletConnectorManager: walletConnectorManager,
            perpetualService: perpetualService,
            hyperliquidObserverService: hyperliquidObserverService,
            nameService: gemNameService,
            toastPresenter: toastPresenter,
            viewModelFactory: viewModelFactory,
            rewardsService: rewardsService,
            searchService: searchService,
            appLifecycleService: appLifecycleService,
            inAppNotificationService: inAppNotificationService,
            portfolioService: portfolioService,
            fiatService: fiatService,
            contactService: contactService,
            supportChatService: supportChatService,
        )
    }
}

// MARK: - Private Static

extension ServicesFactory {
    private static func makeDeviceApiClient(
        provider: NativeProvider,
        devicePrivateKey: Data,
        preflight: (any GemWalletRequestPreflight)? = nil,
    ) -> Gemstone.GemDeviceApiClient {
        if let preflight {
            return Gemstone.GemDeviceApiClient.withPreflight(
                provider: provider,
                baseUrl: Constants.apiURL.absoluteString,
                devicePrivateKey: devicePrivateKey,
                preflight: preflight,
            )
        }
        return Gemstone.GemDeviceApiClient(
            provider: provider,
            baseUrl: Constants.apiURL.absoluteString,
            devicePrivateKey: devicePrivateKey,
        )
    }

    private static func makeDeviceObserverService(
        deviceService: any DeviceServiceable,
        walletStore: WalletStore,
    ) -> DeviceObserverService {
        DeviceObserverService(
            deviceService: deviceService,
            subscriptionsObserver: walletStore.observer(),
        )
    }

    private static func makeTransactionService(
        transactionStore: TransactionStore,
        gatewayService: GatewayService,
        stakeService: any GemStakeServiceProtocol,
        nftService: any GemNftServiceProtocol,
        balanceService: any GemBalanceServiceProtocol,
    ) -> TransactionStateScheduler {
        let postProcessingService = TransactionPostProcessingService(
            balanceService: balanceService,
            stakeService: stakeService,
            nftService: nftService,
        )
        let service = TransactionStateService(
            service: gatewayService.transactionStateService(store: GemstoneTransactionStateStore(store: transactionStore)),
            postProcessingService: postProcessingService,
        )
        return TransactionStateScheduler(service: service)
    }

    private static func makeConnectionsService(
        connectionsStore: ConnectionsStore,
        walletSessionService: WalletSessionService,
        interactor: any WalletConnectorInteractable,
        transactionSimulationService: TransactionSimulationService,
    ) -> ConnectionsService {
        let signer = WalletConnectorSigner(
            connectionsStore: connectionsStore,
            walletSessionService: walletSessionService,
            walletConnectorInteractor: interactor,
        )
        return ConnectionsService(
            store: connectionsStore,
            connector: WalletConnectorService(
                signer: signer,
                service: GemWalletConnectService(simulation: transactionSimulationService, signer: signer),
            ),
        )
    }

    private static func makeWebSocket(securePreferences: SecurePreferences) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(securePreferences: securePreferences)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider)
        return WebSocketConnection(configuration: configuration)
    }
}
