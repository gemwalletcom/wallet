// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import AppService
import AssetsService
import AvatarService
import BalanceService
import BannerService
import ChainService
import ConnectionsService
import ConnectionStatusService
import ContactService
import DeviceService
import DiscoverAssetsService
import ExplorerService
import FiatService
import Foundation
import NFTService
import NodeService
import NotificationService
import PerpetualService
import PriceAlertService
import PriceService
import Primitives
import PrimitivesComponents
import RewardsService
import ServiceStatusService
import StakeService
import StreamService
import SupportChatService
import SwapService
import TransactionsService
import TransactionStateService
import WalletConnector
import WalletService
import WalletSessionService

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
        let serviceStatusService: any ServiceStatusServiceable
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let priceAlertService: PriceAlertService
        let streamObserverService: StreamObserverService
        let streamSubscriptionService: StreamSubscriptionService
        let priceService: PriceService
        let chartService: ChartService
        let marketService: MarketService
        let stakeService: StakeService
        let transactionsService: TransactionsService
        let transactionStateScheduler: TransactionStateScheduler
        let walletService: WalletService
        let walletSessionService: any WalletSessionManageable
        let assetsEnabler: any AssetsEnabler
        let assetDiscoveryService: any AssetDiscoverable
        let walletSetupService: WalletSetupService
        let explorerService: ExplorerService
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
        let inAppNotificationService: InAppNotificationService
        let portfolioService: PortfolioService
        let fiatService: FiatService
        let contactService: ContactService
        let supportChatService: SupportChatService
    }
}
