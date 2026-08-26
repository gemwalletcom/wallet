// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import GemstoneServices
import AppService
import AvatarService
import BalanceService
import Blockchain
import ChainService
import ConnectionsService
import ConnectionStatusService
import EarnService
import ExplorerService
import Foundation
import GemAPI
import GemAPIDevice
import Gemstone
import GemstonePrimitives
import GemstoneStore
import Keystore
import NativeProviderService
import NodeService
import PerpetualService
import Preferences
import Primitives
import PrimitivesComponents
import ServiceStatusService
import StakeService
import Store
import StreamService
import SwapService
import SwiftHTTPClient
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
        let preferencesService = Gemstone.GemPreferencesService(store: GemstonePreferencesStore(namespace: "gemstone_"))
        let nodeService = NodeService(
            nodeStore: storeManager.nodeStore,
            service: GemNodeService(store: GemstoneNodeStore(store: storeManager.nodeStore)),
        )
        let nativeProvider = NativeProvider(nodeProvider: nodeService)
        let deviceRegistrationClient = Self.makeDeviceApiClient(provider: nativeProvider, securePreferences: securePreferences)

        let gemWalletStore = GemstoneWalletStore(store: storeManager.walletStore)
        let gemDeviceService = Gemstone.GemDeviceService(
            api: deviceRegistrationClient,
            subscriptions: Gemstone.GemSubscriptionService(api: deviceRegistrationClient, store: gemWalletStore),
            walletStore: gemWalletStore,
            store: GemstoneDeviceStore(),
        )
        let deviceService = DeviceService(
            deviceProvider: gemDeviceService,
            preferencesService: preferencesService,
            securePreferences: securePreferences,
        )
        let gemDeviceApiClient = Self.makeDeviceApiClient(
            provider: nativeProvider,
            securePreferences: securePreferences,
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
        let chartService = ChartService(service: Gemstone.GemChartService(api: gemApiClient))
        let gemPriceService = Gemstone.GemPriceService(
            api: gemApiClient,
            store: GemstonePriceStore(priceStore: storeManager.priceStore, fiatRateStore: storeManager.fiatRateStore),
        )
        let marketService = MarketService(service: gemPriceService)
        let priceService = PriceService(priceStore: storeManager.priceStore, service: gemPriceService)
        let gemAssetStore = GemstoneAssetStore(assetStore: storeManager.assetStore, balanceStore: storeManager.balanceStore)
        let gemAssetsService = Gemstone.GemAssetsService(api: gemApiClient, store: gemAssetStore, price: gemPriceService, preferences: preferencesService)
        let gemTransactionsService = Gemstone.GemTransactionsService(
            api: gemDeviceApiClient,
            assets: gemAssetsService,
            store: GemstoneTransactionStore(store: storeManager.transactionStore),
            addressStore: GemstoneAddressStore(store: storeManager.addressStore),
        )
        let gemScanService = Gemstone.GemScanService(api: gemDeviceApiClient)
        let gatewayService = GatewayService(provider: nativeProvider)
        let paymentService = PaymentService(provider: nativeProvider)
        let transactionSimulationService = TransactionSimulationService(provider: nativeProvider)
        let serviceStatusService = ServiceStatusService()
        let chainServiceFactory = ChainServiceFactory(gatewayService: gatewayService)

        let avatarService = AvatarService(store: storeManager.walletStore)
        let assetsService = AssetsService(
            assetStore: storeManager.assetStore,
            balanceStore: storeManager.balanceStore,
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
        let gemBalanceService = gatewayService.balanceService(
            walletStore: gemWalletStore,
            assetStore: gemAssetStore,
            store: GemstoneBalanceStore(store: storeManager.balanceStore),
            assets: gemAssetsService,
            price: gemPriceService,
        )
        let balanceService = BalanceService(balanceStore: storeManager.balanceStore, service: gemBalanceService)
        let gemStakeService = gatewayService.stakeService(
            staticApi: gemStaticApiClient,
            store: GemstoneStakeStore(store: storeManager.stakeStore, addressStore: storeManager.addressStore),
        )
        let earnService = EarnService(store: storeManager.stakeStore, service: gemStakeService, gatewayService: gatewayService)
        let stakeService = StakeService(store: storeManager.stakeStore, service: gemStakeService)
        let gemNftService = Gemstone.GemNftService(api: gemDeviceApiClient, store: GemstoneNftStore(store: storeManager.nftStore))
        let nftService = NFTService(service: gemNftService)
        let transactionsService = TransactionsService(
            service: gemTransactionsService,
            transactionStore: storeManager.transactionStore,
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
        let portfolioService = PortfolioService(service: Gemstone.GemPortfolioService(api: gemDeviceApiClient), assetStore: storeManager.assetStore)
        let gemPerpetualStore = GemstonePerpetualStore(store: storeManager.perpetualStore, assetStore: storeManager.assetStore, balanceStore: storeManager.balanceStore)
        let gemPerpetualService = gatewayService.perpetualService(price: gemPriceService, store: gemPerpetualStore)
        let perpetualService = PerpetualService(
            store: storeManager.perpetualStore,
            perpetualStore: gemPerpetualStore,
            balanceStore: storeManager.balanceStore,
            provider: PerpetualProviderFactory(nodeProvider: nodeProvider).createProvider(),
            service: gemPerpetualService,
            preferences: preferences,
        )
        let webSocket = Self.makeWebSocket(securePreferences: securePreferences)
        let streamSubscriptionService = StreamSubscriptionService(
            priceService: priceService,
            walletSessionService: walletSessionService,
            webSocket: webSocket,
        )
        let gemPriceAlertService = Gemstone.GemPriceAlertService(api: gemDeviceApiClient, preferences: preferencesService, store: GemstonePriceAlertStore(store: storeManager.priceAlertStore))
        let priceAlertService = Self.makePriceAlertService(
            service: gemPriceAlertService,
            priceAlertStore: storeManager.priceAlertStore,
            deviceService: deviceService,
            priceUpdater: streamSubscriptionService,
            preferences: preferences,
        )
        let gemFiatService = Gemstone.GemFiatService(
            api: gemDeviceApiClient,
            assets: gemAssetsService,
            store: GemstoneFiatStore(store: storeManager.fiatTransactionStore),
        )
        let fiatService = FiatService(service: gemFiatService)
        let supportTypingState = SupportTypingState()
        let gemSupportStore = GemstoneSupportStore(store: storeManager.supportChatStore)
        let supportChatService = SupportChatService(
            provider: Gemstone.GemSupportService(api: gemDeviceApiClient, store: gemSupportStore),
            typing: supportTypingState,
        )
        let streamEventService = StreamEventService(
            service: Gemstone.GemStreamService(
                price: gemPriceService,
                priceAlert: gemPriceAlertService,
                balance: gemBalanceService,
                transactions: gemTransactionsService,
                nft: gemNftService,
                perpetual: gemPerpetualService,
                fiat: gemFiatService,
                notifications: GemstoneNotificationStore(store: storeManager.inAppNotificationStore),
                support: gemSupportStore,
                walletStore: gemWalletStore,
            ),
            typing: supportTypingState,
            preferences: preferences,
        )
        let streamObserverService = StreamObserverService(
            subscriptionService: streamSubscriptionService,
            eventService: streamEventService,
            webSocket: webSocket,
        )
        let explorerService = ExplorerService.standard
        let swapService = SwapService(nodeProvider: nodeProvider)

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
        let assetDiscoveryService = AssetDiscoveryService(
            discovery: Gemstone.GemAssetDiscoveryService(
                api: gemDeviceApiClient,
                balance: gemBalanceService,
                transactions: gemTransactionsService,
                nft: gemNftService,
                walletStore: gemWalletStore,
                store: GemstoneAssetDiscoveryStore(),
            ),
            preferences: preferences,
        )
        let walletSetupService = WalletSetupService(balanceService: balanceService)

        let gemConfigService = Gemstone.GemConfigService(api: gemApiClient, preferences: preferencesService)
        let configService = ConfigService(service: gemConfigService)
        let releaseAlertService = ReleaseAlertService(
            appUpdateService: Gemstone.GemAppUpdateService(config: gemConfigService, preferences: preferencesService),
        )
        let rateService = RateService(preferences: preferences)

        let onStartService = OnstartService(
            assetStore: storeManager.assetStore,
            nodeStore: storeManager.nodeStore,
            preferences: preferences,
            preferencesService: preferencesService,
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
            walletConfigurationService: Gemstone.GemWalletConfigurationService(
                api: gemDeviceApiClient,
                banners: GemstoneBannerStore(store: storeManager.bannerStore),
                store: GemstoneWalletConfigurationStore(),
            ),
            pushNotificationEnablerService: pushNotificationEnablerService,
        )

        let hyperliquidObserverService = HyperliquidObserverService(
            nodeProvider: PerpetualNodeService(nodeProvider: nodeProvider),
            perpetualService: perpetualService,
        )

        let gemNameService = Gemstone.GemNameService(api: gemDeviceApiClient, store: GemstoneAddressStore(store: storeManager.addressStore))
        let addressNameService = AddressNameService(addressStore: storeManager.addressStore, service: gemNameService)
        let activityService = ActivityService(store: storeManager.recentActivityStore)
        let authService = AuthService(service: Gemstone.GemAuthService(api: gemDeviceApiClient), keystore: storages.keystore)
        let rewardsService = RewardsService(service: Gemstone.GemRewardsService(api: gemDeviceApiClient), authService: authService)
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
            priceService: priceService,
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

        let contactService = ContactService(
            provider: Gemstone.GemContactService(
                store: GemstoneContactStore(store: storeManager.contactStore),
                addressStore: GemstoneAddressStore(store: storeManager.addressStore),
            ),
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
            releaseAlertService: releaseAlertService,
            rateService: rateService,
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
            service: gatewayService.transactionStateService(store: GemstoneTransactionStateStore(store: transactionStore)),
            postProcessingService: postProcessingService,
        )
        return TransactionStateScheduler(
            transactionStore: transactionStore,
            service: service,
        )
    }

    private static func makePriceAlertService(
        service: any Gemstone.GemPriceAlertServiceProtocol,
        priceAlertStore: PriceAlertStore,
        deviceService: any DeviceServiceable,
        priceUpdater: any PriceUpdater,
        preferences: Preferences,
    ) -> PriceAlertService {
        PriceAlertService(
            store: priceAlertStore,
            service: service,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
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
        return OnstartAsyncService(
            runners: [
                ConfigUpdateRunner(configService: configService),
                BannerSetupRunner(bannerSetupService: bannerSetupService),
                AssetsUpdateRunner(
                    configService: configService,
                    assetsProvider: assetsProvider,
                    assetsService: assetsService,
                    swappableChainsProvider: swappableChainsProvider,
                ),
            ],
        )
    }


    private static func makeWebSocket(securePreferences: SecurePreferences) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(securePreferences: securePreferences)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider)
        return WebSocketConnection(configuration: configuration)
    }
}
