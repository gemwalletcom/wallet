// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletPreferencesServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import protocol Gemstone.GemContactServiceProtocol
import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemAvatarServiceProtocol
import class Gemstone.GemStreamSubscriptionService
import protocol Gemstone.GemPriceAlertServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemAppStartServiceProtocol
import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemRewardsServiceProtocol
import protocol Gemstone.GemSearchServiceProtocol
import protocol Gemstone.GemSwapServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemFiatServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import AppService
import ConnectionsService
import ConnectionStatusService
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemChartServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import protocol Gemstone.GemNotificationServiceProtocol
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import Primitives
import PrimitivesComponents
import protocol Gemstone.GemServiceStatusProtocol
import StreamService
import WalletConnector

extension AppResolver {
    struct Services {
        // Environment-level services
        let balanceService: any GemBalanceServiceProtocol
        let bannerService: any GemBannerServiceProtocol
        let chainServiceFactory: ChainServiceFactory
        let connectionsService: ConnectionsService
        let connectionStatusObserver: ConnectionStatusObserver
        let deviceService: DeviceService
        let nodeService: NodeService
        let serviceStatusService: any GemServiceStatusProtocol
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let priceAlertService: any GemPriceAlertServiceProtocol
        let streamObserverService: StreamObserverService
        let streamSubscriptionService: GemStreamSubscriptionService
        let priceService: any GemPriceServiceProtocol
        let chartService: any GemChartServiceProtocol
        let marketService: any GemPriceServiceProtocol
        let stakeService: any GemStakeServiceProtocol
        let transactionsService: any GemTransactionsServiceProtocol
        let transactionStateScheduler: TransactionStateScheduler
        let walletService: WalletService
        let walletPreferencesService: any GemWalletPreferencesServiceProtocol
        let walletSessionService: any WalletSessionManageable
        let assetsEnabler: any AssetsEnabler
        let assetDiscoveryService: any GemAssetDiscoveryServiceProtocol
        let gemAssetsService: any GemAssetsServiceProtocol
        let explorerService: any GemExplorerServiceProtocol
        let gatewayService: GatewayService
        let nftService: any GemNftServiceProtocol
        let avatarService: any GemAvatarServiceProtocol
        let swapService: any GemSwapServiceProtocol
        let appUpdateService: any GemAppUpdateServiceProtocol
        let rateService: RateService
        let deviceObserverService: DeviceObserverService
        let onstartService: OnstartService
        let appStartService: any GemAppStartServiceProtocol
        let pushNotificationEnablerService: PushNotificationEnablerService
        let walletConnectorManager: WalletConnectorManager
        let perpetualService: PerpetualService
        let hyperliquidObserverService: any PerpetualObservable
        let nameService: any GemNameServiceProtocol
        let toastPresenter: ToastPresenter
        let viewModelFactory: ViewModelFactory
        let rewardsService: any GemRewardsServiceProtocol
        let searchService: any GemSearchServiceProtocol
        let appLifecycleService: AppLifecycleService
        let inAppNotificationService: any GemNotificationServiceProtocol
        let portfolioService: any GemPortfolioServiceProtocol
        let fiatService: any GemFiatServiceProtocol
        let contactService: any GemContactServiceProtocol
        let supportService: any GemSupportServiceProtocol
        let supportTypingState: SupportTypingState
    }
}
