// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import ActivityService
import AppService
import GemstoneServices
import ConnectionsService
import ConnectionStatusService
import Foundation
import protocol Gemstone.GemChartServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import protocol Gemstone.GemNotificationServiceProtocol
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GRDB
import Primitives
import protocol Gemstone.GemServiceStatusProtocol
import Store
import StreamService
import SwiftUI
import WalletConnector

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var nodeService: NodeService = AppResolver.main.services.nodeService
    @Entry var serviceStatusService: any GemServiceStatusProtocol = AppResolver.main.services.serviceStatusService
    @Entry var priceService: any GemPriceServiceProtocol = AppResolver.main.services.priceService
    @Entry var priceStore: PriceStore = StoreManager(db: AppResolver.main.storages.db).priceStore
    @Entry var chartService: any GemChartServiceProtocol = AppResolver.main.services.chartService
    @Entry var marketService: any GemPriceServiceProtocol = AppResolver.main.services.marketService
    @Entry var streamSubscriptionService: StreamSubscriptionService = AppResolver.main.services.streamSubscriptionService
    @Entry var assetsEnabler: any AssetsEnabler = AppResolver.main.services.assetsEnabler
    @Entry var assetDiscoveryService: any GemAssetDiscoveryServiceProtocol = AppResolver.main.services.assetDiscoveryService
    @Entry var walletService: WalletService = AppResolver.main.services.walletService
    @Entry var walletSessionService: any WalletSessionManageable = AppResolver.main.services.walletSessionService
    @Entry var priceAlertService: PriceAlertService = AppResolver.main.services.priceAlertService
    @Entry var deviceService: DeviceService = AppResolver.main.services.deviceService
    @Entry var balanceService: any GemBalanceServiceProtocol = AppResolver.main.services.balanceService
    @Entry var bannerService: BannerService = AppResolver.main.services.bannerService
    @Entry var transactionsService: any GemTransactionsServiceProtocol = AppResolver.main.services.transactionsService
    @Entry var transactionStore: TransactionStore = StoreManager(db: AppResolver.main.storages.db).transactionStore
    @Entry var assetsService: AssetsService = AppResolver.main.services.assetsService
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationHandler: NavigationHandler = AppResolver.main.services.navigationHandler
    @Entry var stakeService: any GemStakeServiceProtocol = AppResolver.main.services.stakeService
    @Entry var stakeStore: StakeStore = StoreManager(db: AppResolver.main.storages.db).stakeStore
    @Entry var explorerService: any GemExplorerServiceProtocol = AppResolver.main.services.explorerService
    @Entry var gatewayService: GatewayService = AppResolver.main.services.gatewayService
    @Entry var connectionsService: ConnectionsService = AppResolver.main.services.connectionsService
    @Entry var connectionStatus: ConnectionStatusObserver = AppResolver.main.services.connectionStatusObserver
    @Entry var walletConnectorManager: WalletConnectorManager = AppResolver.main.services.walletConnectorManager
    @Entry var chainServiceFactory: ChainServiceFactory = AppResolver.main.services.chainServiceFactory
    @Entry var nftService: any GemNftServiceProtocol = AppResolver.main.services.nftService
    @Entry var avatarService: AvatarService = AppResolver.main.services.avatarService
    @Entry var releaseAlertService: ReleaseAlertService = AppResolver.main.services.releaseAlertService
    @Entry var perpetualService: PerpetualService = AppResolver.main.services.perpetualService
    @Entry var hyperliquidObserverService: any PerpetualObservable = AppResolver.main.services.hyperliquidObserverService
    @Entry var nameService: any NameServiceable = AppResolver.main.services.nameService
    @Entry var activityService: ActivityService = AppResolver.main.services.activityService
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
    @Entry var rewardsService: RewardsService = AppResolver.main.services.rewardsService
    @Entry var walletSearchService: WalletSearchService = AppResolver.main.services.walletSearchService
    @Entry var inAppNotificationService: any GemNotificationServiceProtocol = AppResolver.main.services.inAppNotificationService
    @Entry var portfolioService: any GemPortfolioServiceProtocol = AppResolver.main.services.portfolioService
    @Entry var contactService: ContactService = AppResolver.main.services.contactService
    @Entry var supportChatService: SupportChatService = AppResolver.main.services.supportChatService
}
