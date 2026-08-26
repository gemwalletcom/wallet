// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import GemstoneServices
import AppService
import AvatarService
import ConnectionsService
import ConnectionStatusService
import protocol Gemstone.GemExplorerServiceProtocol
import Foundation
import protocol Gemstone.GemChartServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import protocol Gemstone.GemNotificationServiceProtocol
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import Primitives
import PrimitivesComponents
import protocol Gemstone.GemServiceStatusProtocol
import StreamService
import SwapService
import WalletConnector
import WalletService

extension AppResolver {
    struct Services {
        // Environment-level services
        let assetsService: AssetsService
        let balanceService: BalanceService
        let bannerService: BannerService
        let chainServiceFactory: ChainServiceFactory
        let connectionsService: ConnectionsService
        let connectionStatusObserver: ConnectionStatusObserver
        let deviceService: DeviceService
        let nodeService: NodeService
        let serviceStatusService: any GemServiceStatusProtocol
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let priceAlertService: PriceAlertService
        let streamObserverService: StreamObserverService
        let streamSubscriptionService: StreamSubscriptionService
        let priceService: PriceService
        let chartService: any GemChartServiceProtocol
        let marketService: any GemPriceServiceProtocol
        let stakeService: any GemStakeServiceProtocol
        let transactionsService: TransactionsService
        let transactionStateScheduler: TransactionStateScheduler
        let walletService: WalletService
        let walletSessionService: any WalletSessionManageable
        let assetsEnabler: any AssetsEnabler
        let assetDiscoveryService: any GemAssetDiscoveryServiceProtocol
        let walletSetupService: WalletSetupService
        let explorerService: any GemExplorerServiceProtocol
        let gatewayService: GatewayService
        let nftService: NFTService
        let avatarService: AvatarService
        let swapService: SwapService
        let releaseAlertService: ReleaseAlertService
        let rateService: RateService
        let deviceObserverService: DeviceObserverService
        let onstartService: OnstartService
        let onstartAsyncService: OnstartAsyncService
        let onstartWalletService: OnstartWalletService
        let walletConnectorManager: WalletConnectorManager
        let perpetualService: PerpetualService
        let hyperliquidObserverService: any PerpetualObservable
        let nameService: any NameServiceable
        let addressNameService: AddressNameService
        let activityService: ActivityService
        let toastPresenter: ToastPresenter
        let viewModelFactory: ViewModelFactory
        let rewardsService: RewardsService
        let walletSearchService: WalletSearchService
        let assetSearchService: AssetSearchService
        let appLifecycleService: AppLifecycleService
        let inAppNotificationService: any GemNotificationServiceProtocol
        let portfolioService: PortfolioService
        let fiatService: FiatService
        let contactService: ContactService
        let supportChatService: SupportChatService
    }
}
