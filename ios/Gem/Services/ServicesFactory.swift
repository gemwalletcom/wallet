// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import AppService
import AssetsService
import AuthService
import AvatarService
import BalanceService
import BannerService
import Blockchain
import ChainService
import ConnectionsService
import ConnectionStatusService
import ContactService
import DeviceService
import DiscoverAssetsService
import EarnService
import ExplorerService
import FiatService
import Foundation
import GemAPI
import GemAPIDevice
import Gemstone
import GemstonePrimitives
import Keystore
import NativeProviderService
import NFTService
import NodeService
import NotificationService
import PerpetualService
import Preferences
import PriceAlertService
import PriceService
import Primitives
import PrimitivesComponents
import RewardsService
import ServiceStatusService
import StakeService
import Store
import StreamService
import SupportChatService
import SwapService
import SwiftHTTPClient
import TransactionsService
import TransactionStateService
import Transfer
import WalletConnector
import WalletConnectorService
import WalletService
import WalletSessionService
import WebSocketClient

struct ServicesFactory {
    func makeServices(storages: AppResolver.Storages, navigation: NavigationStateManager) -> AppResolver.Services {
        let storeManager = StoreManager(db: storages.db)
        let securePreferences = SecurePreferences()
        let nodeService = NodeService(nodeStore: storeManager.nodeStore)
        let nodeAuthProvider = NodeAuthTokenProvider(securePreferences: securePreferences)
        let nativeProvider = NativeProvider(nodeProvider: nodeService, requestInterceptor: nodeAuthProvider)
        let deviceRegistrationClient = Self.makeDeviceApiClient(provider: nativeProvider, securePreferences: securePreferences)

        let subscriptionService = SubscriptionService(
            subscriptionProvider: Gemstone.GemSubscriptionService(api: deviceRegistrationClient),
            walletStore: storeManager.walletStore,
        )
        let deviceService = DeviceService(
            deviceProvider: Gemstone.GemDeviceService(api: deviceRegistrationClient),
            subscriptionsService: subscriptionService,
            securePreferences: securePreferences,
        )
        let gemDeviceApiClient = Self.makeDeviceApiClient(
            provider: nativeProvider,
            securePreferences: securePreferences,
            preflight: DeviceSyncPreflight(deviceService: deviceService),
        )
        let gemTransactionsService = Gemstone.GemTransactionsService(api: gemDeviceApiClient)
        let deviceObserverService = Self.makeDeviceObserverService(
            deviceService: deviceService,
            subscriptionService: subscriptionService,
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
        let chartService = ChartService(service: Gemstone.GemChartService(api: gemApiClient))
        let marketService = MarketService(service: Gemstone.GemPriceService(api: gemApiClient))
        let staticAssetsService = Gemstone.GemStaticAssetsService(api: gemStaticApiClient)
        let gemAssetsService = Gemstone.GemAssetsService(api: gemApiClient)
        let gemScanService = Gemstone.GemScanService(api: gemDeviceApiClient)
        let gatewayService = GatewayService(provider: nativeProvider)
        let paymentService = PaymentService(provider: nativeProvider)
        let transactionSimulationService = TransactionSimulationService(provider: nativeProvider)
        let serviceStatusService = ServiceStatusService(requestInterceptor: nodeAuthProvider)
        let chainServiceFactory = ChainServiceFactory(
            gatewayService: gatewayService,
            requestInterceptor: nodeAuthProvider,
        )

        let avatarService = AvatarService(store: storeManager.walletStore)
        let assetsService = AssetsService(
            assetStore: storeManager.assetStore,
            balanceStore: storeManager.balanceStore,
            priceStore: storeManager.priceStore,
            chainServiceFactory: chainServiceFactory,
            assetsProvider: gemAssetsService,
        )

        let walletSessionService = WalletSessionService(
            walletStore: storeManager.walletStore,
            preferences: storages.observablePreferences,
        )
        let walletService = WalletService(
            keystore: storages.keystore,
            walletStore: storeManager.walletStore,
            preferences: storages.observablePreferences,
            avatarService: avatarService,
            walletSessionService: walletSessionService,
        )
        let balanceService = BalanceService(
            balanceStore: storeManager.balanceStore,
            assetsService: assetsService,
            chainServiceFactory: chainServiceFactory,
        )
        let earnService = EarnService(
            store: storeManager.stakeStore,
            gatewayService: gatewayService,
        )
        let stakeService = StakeService(
            store: storeManager.stakeStore,
            addressStore: storeManager.addressStore,
            chainServiceFactory: chainServiceFactory,
            assetsService: staticAssetsService,
        )
        let nftService = NFTService(
            service: Gemstone.GemNftService(api: gemDeviceApiClient),
            nftStore: storeManager.nftStore,
        )
        let transactionsService = TransactionsService(
            provider: gemTransactionsService,
            transactionStore: storeManager.transactionStore,
            assetsService: assetsService,
            addressStore: storeManager.addressStore,
        )
        let transactionStateScheduler = Self.makeTransactionService(
            transactionStore: storeManager.transactionStore,
            gatewayService: gatewayService,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
            balanceService: balanceService,
        )

        let preferences = storages.observablePreferences.preferences
        let pushNotificationEnablerService = PushNotificationEnablerService(preferences: preferences)
        let bannerSetupService = BannerSetupService(store: storeManager.bannerStore, preferences: preferences)
        let bannerService = BannerService(
            store: storeManager.bannerStore,
            service: Gemstone.GemBannerService(store: GemstoneBannerStore(store: storeManager.bannerStore)),
            pushNotificationService: pushNotificationEnablerService,
        )
        let navigationPresenter = NavigationPresenter()
        let priceService = PriceService(
            priceStore: storeManager.priceStore,
            fiatRateStore: storeManager.fiatRateStore,
        )
        let portfolioService = PortfolioService(apiService: Gemstone.GemPortfolioService(api: gemDeviceApiClient), assetStore: storeManager.assetStore)
        let perpetualService = Self.makePerpetualService(
            perpetualStore: storeManager.perpetualStore,
            assetStore: storeManager.assetStore,
            priceStore: storeManager.priceStore,
            balanceStore: storeManager.balanceStore,
            nodeProvider: nodeProvider,
            requestInterceptor: nodeAuthProvider,
            preferences: preferences,
        )
        let webSocket = Self.makeWebSocket(securePreferences: securePreferences)
        let streamSubscriptionService = StreamSubscriptionService(
            priceService: priceService,
            walletSessionService: walletSessionService,
            webSocket: webSocket,
        )
        let priceAlertService = Self.makePriceAlertService(
            apiService: Gemstone.GemPriceAlertService(api: gemDeviceApiClient),
            priceAlertStore: storeManager.priceAlertStore,
            deviceService: deviceService,
            priceUpdater: streamSubscriptionService,
            preferences: preferences,
        )
        let fiatService = FiatService(
            apiService: Gemstone.GemFiatService(api: gemDeviceApiClient),
            assetsService: assetsService,
            store: storeManager.fiatTransactionStore,
        )
        let supportTypingState = SupportTypingState()
        let supportChatService = SupportChatService(store: storeManager.supportChatStore, provider: Gemstone.GemSupportService(api: gemDeviceApiClient), typing: supportTypingState)
        let streamEventService = StreamEventService(
            walletStore: storeManager.walletStore,
            notificationStore: storeManager.inAppNotificationStore,
            priceService: priceService,
            priceAlertService: priceAlertService,
            balanceUpdater: balanceService,
            transactionsService: transactionsService,
            nftService: nftService,
            perpetualService: perpetualService,
            fiatService: fiatService,
            supportChatService: supportChatService,
            preferences: preferences,
        )
        let streamObserverService = StreamObserverService(
            subscriptionService: streamSubscriptionService,
            eventService: streamEventService,
            webSocket: webSocket,
        )
        let explorerService = ExplorerService.standard
        let swapService = SwapService(nodeProvider: nodeProvider, requestInterceptor: nodeAuthProvider)

        let presenter = WalletConnectorPresenter()
        let walletConnectorManager = WalletConnectorManager(presenter: presenter)
        let connectionsService = Self.makeConnectionsService(
            connectionsStore: storeManager.connectionsStore,
            walletSessionService: walletSessionService,
            interactor: walletConnectorManager,
            transactionSimulationService: transactionSimulationService,
        )

        let assetsEnabler = AssetsEnablerService(
            assetsService: assetsService,
            balanceUpdater: balanceService,
            priceUpdater: streamSubscriptionService,
        )
        let assetDiscoveryService = AssetDiscoveryService(
            assetsListService: gemTransactionsService,
            assetService: assetsService,
            assetsEnabler: assetsEnabler,
            transactionsService: transactionsService,
            nftService: nftService,
        )
        let walletSetupService = WalletSetupService(balanceService: balanceService)

        let configService = ConfigService(service: Gemstone.GemConfigService(api: gemApiClient))
        let releaseService = AppReleaseService(configService: configService)
        let releaseAlertService = ReleaseAlertService(
            appReleaseService: releaseService,
            preferences: preferences,
        )
        let rateService = RateService(preferences: preferences)

        let onStartService = OnstartService(
            assetsProvider: gemAssetsService,
            assetsService: assetsService,
            assetStore: storeManager.assetStore,
            nodeStore: storeManager.nodeStore,
            preferences: preferences,
            walletService: walletService,
        )
        let onstartAsyncService = Self.makeOnstartAsyncService(
            assetsProvider: gemAssetsService,
            nodeService: nodeService,
            preferences: preferences,
            assetsService: assetsService,
            bannerSetupService: bannerSetupService,
            configService: configService,
            swappableChainsProvider: swapService,
        )
        let onstartWalletService = OnstartWalletService(
            deviceService: deviceService,
            bannerSetupService: bannerSetupService,
            walletConfigurationService: Gemstone.GemWalletConfigurationService(api: gemDeviceApiClient),
            pushNotificationEnablerService: pushNotificationEnablerService,
        )

        let hyperliquidObserverService = HyperliquidObserverService(
            nodeProvider: PerpetualNodeService(nodeProvider: nodeProvider),
            perpetualService: perpetualService,
        )

        let gemNameService = Gemstone.GemNameService(api: gemDeviceApiClient)
        let addressNameService = AddressNameService(addressStore: storeManager.addressStore, apiService: gemNameService)
        let activityService = ActivityService(store: storeManager.recentActivityStore)
        let authService = AuthService(apiService: Gemstone.GemAuthService(api: gemDeviceApiClient), keystore: storages.keystore)
        let rewardsService = RewardsService(apiService: Gemstone.GemRewardsService(api: gemDeviceApiClient), authService: authService)
        let toastPresenter = ToastPresenter()
        let navigationHandler = NavigationHandler(
            navigationState: navigation,
            presenter: navigationPresenter,
            assetsService: assetsService,
            connectionsService: connectionsService,
            toastPresenter: toastPresenter,
            paymentService: paymentService,
            transactionsService: transactionsService,
            walletConnectorPresenter: presenter,
            walletSessionService: walletSessionService,
        )
        let walletSearchService = WalletSearchService(
            assetsService: assetsService,
            searchStore: storeManager.searchStore,
            perpetualStore: storeManager.perpetualStore,
            assetListStore: storeManager.assetListStore,
            priceStore: storeManager.priceStore,
            preferences: preferences,
            searchProvider: gemAssetsService,
        )
        let assetSearchService = AssetSearchService(
            assetsService: assetsService,
            searchStore: storeManager.searchStore,
        )
        let inAppNotificationService = InAppNotificationService(
            service: Gemstone.GemNotificationService(
                api: gemDeviceApiClient,
                store: GemstoneNotificationStore(store: storeManager.inAppNotificationStore),
            ),
        )

        let contactService = ContactService(store: storeManager.contactStore, addressStore: storeManager.addressStore)

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
            earnService: earnService,
            amountService: AmountService(earnDataProvider: earnService),
            nameService: GemstoneNameService(service: gemNameService),
            balanceService: balanceService,
            priceService: priceService,
            transactionStateScheduler: transactionStateScheduler,
            addressNameService: addressNameService,
            activityService: activityService,
            toastPresenter: toastPresenter,
            fiatService: fiatService,
            assetsService: assetsService,
            assetSearchService: assetSearchService,
            priceAlertService: priceAlertService,
            walletSearchService: walletSearchService,
            perpetualService: perpetualService,
        )

        return AppResolver.Services(
            assetsService: assetsService,
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
            walletSetupService: walletSetupService,
            explorerService: explorerService,
            nftService: nftService,
            avatarService: avatarService,
            swapService: swapService,
            appReleaseService: releaseService,
            releaseAlertService: releaseAlertService,
            rateService: rateService,
            subscriptionsService: subscriptionService,
            deviceObserverService: deviceObserverService,
            onstartService: onStartService,
            onstartAsyncService: onstartAsyncService,
            onstartWalletService: onstartWalletService,
            walletConnectorManager: walletConnectorManager,
            perpetualService: perpetualService,
            hyperliquidObserverService: hyperliquidObserverService,
            nameService: GemstoneNameService(service: gemNameService),
            addressNameService: addressNameService,
            activityService: activityService,
            toastPresenter: toastPresenter,
            viewModelFactory: viewModelFactory,
            rewardsService: rewardsService,
            walletSearchService: walletSearchService,
            assetSearchService: assetSearchService,
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
        securePreferences: SecurePreferences,
        preflight: (any GemWalletRequestPreflight)? = nil,
    ) -> Gemstone.GemDeviceApiClient {
        let keyPair = try? DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        let privateKey = keyPair?.privateKey ?? Data()
        if let preflight {
            return Gemstone.GemDeviceApiClient.withPreflight(
                provider: provider,
                baseUrl: Constants.apiURL.absoluteString,
                devicePrivateKey: privateKey,
                preflight: preflight,
            )
        }
        return Gemstone.GemDeviceApiClient(
            provider: provider,
            baseUrl: Constants.apiURL.absoluteString,
            devicePrivateKey: privateKey,
        )
    }

    private static func makeRequestSigner(securePreferences: SecurePreferences) -> DeviceRequestSigner? {
        do {
            let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
            return try DeviceRequestSigner(privateKey: keyPair.privateKey)
        } catch {
            debugLog("makeRequestSigner error: \(error)")
            return nil
        }
    }

    private static func makeDeviceObserverService(
        deviceService: any DeviceServiceable,
        subscriptionService: SubscriptionService,
        walletStore: WalletStore,
    ) -> DeviceObserverService {
        DeviceObserverService(
            deviceService: deviceService,
            subscriptionsService: subscriptionService,
            subscriptionsObserver: walletStore.observer(),
        )
    }

    private static func makeTransactionService(
        transactionStore: TransactionStore,
        gatewayService: GatewayService,
        stakeService: StakeService,
        earnService: EarnService,
        nftService: NFTService,
        balanceService: BalanceService,
    ) -> TransactionStateScheduler {
        let postProcessingService = TransactionPostProcessingService(
            transactionStore: transactionStore,
            balanceUpdater: balanceService,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
        )
        let service = TransactionStateService(
            transactionStore: transactionStore,
            gatewayService: gatewayService,
            postProcessingService: postProcessingService,
        )
        return TransactionStateScheduler(
            transactionStore: transactionStore,
            service: service,
        )
    }

    private static func makePriceAlertService(
        apiService: any Gemstone.GemPriceAlertServiceProtocol,
        priceAlertStore: PriceAlertStore,
        deviceService: any DeviceServiceable,
        priceUpdater: any PriceUpdater,
        preferences: Preferences,
    ) -> PriceAlertService {
        PriceAlertService(
            store: priceAlertStore,
            apiService: apiService,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            preferences: preferences,
            pushNotificationService: PushNotificationEnablerService(preferences: preferences),
        )
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
            signer: signer,
            connector: WalletConnectorService(signer: signer, transactionSimulationService: transactionSimulationService),
        )
    }

    private static func makeOnstartAsyncService(
        assetsProvider: any Gemstone.GemAssetsServiceProtocol,
        nodeService: NodeService,
        preferences: Preferences,
        assetsService: AssetsService,
        bannerSetupService: BannerSetupService,
        configService: ConfigService,
        swappableChainsProvider: any SwappableChainsProvider,
    ) -> OnstartAsyncService {
        let importAssetsService = ImportAssetsService(
            assetsProvider: assetsProvider,
            assetsService: assetsService,
            assetStore: assetsService.assetStore,
            preferences: preferences,
        )

        return OnstartAsyncService(
            runners: [
                ConfigUpdateRunner(configService: configService),
                BannerSetupRunner(bannerSetupService: bannerSetupService),
                NodeImportRunner(nodeService: nodeService),
                AssetsUpdateRunner(
                    configService: configService,
                    importAssetsService: importAssetsService,
                    assetsService: assetsService,
                    swappableChainsProvider: swappableChainsProvider,
                    preferences: preferences,
                ),
            ],
        )
    }

    private static func makePerpetualService(
        perpetualStore: PerpetualStore,
        assetStore: AssetStore,
        priceStore: PriceStore,
        balanceStore: BalanceStore,
        nodeProvider: any NodeURLFetchable,
        requestInterceptor: any RequestInterceptable,
        preferences: Preferences,
    ) -> PerpetualService {
        PerpetualService(
            store: perpetualStore,
            assetStore: assetStore,
            priceStore: priceStore,
            balanceStore: balanceStore,
            provider: PerpetualProviderFactory(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor).createProvider(),
            preferences: preferences,
        )
    }

    private static func makeWebSocket(securePreferences: SecurePreferences) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(securePreferences: securePreferences)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider)
        return WebSocketConnection(configuration: configuration)
    }
}
