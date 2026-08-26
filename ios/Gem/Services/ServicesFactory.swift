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
        let signer = Self.makeRequestSigner(securePreferences: securePreferences)
        let interceptor: (@Sendable (inout URLRequest, GemDeviceAPI) throws -> Void)? = if let signer {
            { request, target in
                try signer.sign(request: &request, walletId: target.walletId ?? "")
            }
        } else {
            nil
        }
        let provider = Provider<GemAPI>()
        let deviceProvider = Provider<GemDeviceAPI>(options: ProviderOptions(baseUrl: nil, requestInterceptor: interceptor))
        let deviceAPIService = GemDeviceService(deviceProvider: deviceProvider)

        let subscriptionService = SubscriptionService(
            subscriptionProvider: deviceAPIService,
            walletStore: storeManager.walletStore,
        )
        let deviceService = DeviceService(
            deviceProvider: deviceAPIService,
            subscriptionsService: subscriptionService,
            securePreferences: securePreferences,
        )
        let apiService = GemAPIService(
            provider: provider,
            deviceProvider: deviceProvider,
            walletRequestPreflight: {
                try await deviceService.synchronizeIfNeeded()
            },
        )
        let deviceObserverService = Self.makeDeviceObserverService(
            deviceService: deviceService,
            subscriptionService: subscriptionService,
            walletStore: storeManager.walletStore,
        )

        let nodeService = NodeService(nodeStore: storeManager.nodeStore)
        let nodeAuthProvider = NodeAuthTokenProvider(securePreferences: securePreferences)
        let nodeProvider: any NodeURLFetchable = nodeService
        let connectionStatusObserver = ConnectionStatusObserver(
            monitors: [
                InternetConnectionMonitor(),
            ],
        )
        let nativeProvider = NativeProvider(nodeProvider: nodeProvider, requestInterceptor: nodeAuthProvider)
        let gemApiClient = Gemstone.GemApiClient(provider: nativeProvider, baseUrl: Constants.apiURL.absoluteString)
        let gemStaticApiClient = Gemstone.GemStaticApiClient(provider: nativeProvider, baseUrl: Constants.assetsURL.absoluteString)
        let chartService = ChartService(service: Gemstone.GemChartService(api: gemApiClient))
        let marketService = MarketService(service: Gemstone.GemPriceService(api: gemApiClient))
        let staticAssetsService = Gemstone.GemStaticAssetsService(api: gemStaticApiClient)
        let gemScanService = Self.makeScanService(provider: nativeProvider, securePreferences: securePreferences)
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
            apiService: apiService,
            nftStore: storeManager.nftStore,
        )
        let transactionsService = TransactionsService(
            provider: apiService,
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
        let portfolioService = PortfolioService(apiService: apiService, assetStore: storeManager.assetStore)
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
            apiService: apiService,
            priceAlertStore: storeManager.priceAlertStore,
            deviceService: deviceService,
            priceUpdater: streamSubscriptionService,
            preferences: preferences,
        )
        let fiatService = FiatService(
            apiService: apiService,
            assetsService: assetsService,
            store: storeManager.fiatTransactionStore,
        )
        let supportTypingState = SupportTypingState()
        let supportChatService = SupportChatService(store: storeManager.supportChatStore, provider: apiService, typing: supportTypingState)
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
            assetsListService: apiService,
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
            assetListService: apiService,
            assetsService: assetsService,
            assetStore: storeManager.assetStore,
            nodeStore: storeManager.nodeStore,
            preferences: preferences,
            walletService: walletService,
        )
        let onstartAsyncService = Self.makeOnstartAsyncService(
            apiService: apiService,
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
            walletConfigurationService: apiService,
            pushNotificationEnablerService: pushNotificationEnablerService,
        )

        let hyperliquidObserverService = HyperliquidObserverService(
            nodeProvider: PerpetualNodeService(nodeProvider: nodeProvider),
            perpetualService: perpetualService,
        )

        let addressNameService = AddressNameService(addressStore: storeManager.addressStore, apiService: apiService)
        let activityService = ActivityService(store: storeManager.recentActivityStore)
        let authService = AuthService(apiService: apiService, keystore: storages.keystore)
        let rewardsService = RewardsService(apiService: apiService, authService: authService)
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
        )
        let assetSearchService = AssetSearchService(
            assetsService: assetsService,
            searchStore: storeManager.searchStore,
        )
        let inAppNotificationService = InAppNotificationService(
            apiService: apiService,
            store: storeManager.inAppNotificationStore,
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
            nameService: apiService,
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
            nameService: apiService,
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
    private static func makeScanService(provider: NativeProvider, securePreferences: SecurePreferences) -> Gemstone.GemScanService {
        let keyPair = try? DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        let client = Gemstone.GemDeviceApiClient(
            provider: provider,
            baseUrl: Constants.apiURL.absoluteString,
            devicePrivateKey: keyPair?.privateKey ?? Data(),
        )
        return Gemstone.GemScanService(api: client)
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
        apiService: GemAPIService,
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
        apiService: GemAPIService,
        nodeService: NodeService,
        preferences: Preferences,
        assetsService: AssetsService,
        bannerSetupService: BannerSetupService,
        configService: ConfigService,
        swappableChainsProvider: any SwappableChainsProvider,
    ) -> OnstartAsyncService {
        let importAssetsService = ImportAssetsService(
            assetListService: apiService,
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
