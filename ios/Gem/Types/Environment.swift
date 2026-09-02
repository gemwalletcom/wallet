// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemAddressServiceProtocol
import class Gemstone.GemApplicationMetadataService
import class Gemstone.GemAssetConfigService
import class Gemstone.GemDeeplinkService
import class Gemstone.GemChainService
import class Gemstone.GemReceiveService
import class Gemstone.GemTransactionFormatter
import protocol Gemstone.GemSwapServiceProtocol
import class Gemstone.GemDeviceKeyService
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemWalletPreferencesServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemAvatarServiceProtocol
import class Gemstone.GemStreamSubscriptionService
import protocol Gemstone.GemPriceAlertServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemRewardsServiceProtocol
import protocol Gemstone.GemSearchServiceProtocol
import protocol Gemstone.GemTransactionsServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import class Gemstone.GemNodeService
import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Preferences
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
    @Entry var nodeService: GemNodeService = AppResolver.main.services.nodeService
    @Entry var serviceStatusService: any GemServiceStatusProtocol = AppResolver.main.services.serviceStatusService
    @Entry var priceService: any GemPriceServiceProtocol = AppResolver.main.services.priceService
    @Entry var chartService: any GemChartServiceProtocol = AppResolver.main.services.chartService
    @Entry var streamSubscriptionService: GemStreamSubscriptionService = AppResolver.main.services.streamSubscriptionService
    @Entry var assetDiscoveryService: any GemAssetDiscoveryServiceProtocol = AppResolver.main.services.assetDiscoveryService
    @Entry var walletPreferencesService: any GemWalletPreferencesServiceProtocol = AppResolver.main.services.walletPreferencesService
    @Entry var observablePreferences: ObservablePreferences = AppResolver.main.services.observablePreferences
    @Entry var preferencesService: any GemPreferencesServiceProtocol = AppResolver.main.services.preferencesService
    @Entry var deviceKeyService: GemDeviceKeyService = AppResolver.main.services.deviceKeyService
    @Entry var walletSessionService: any GemWalletSessionServiceProtocol = AppResolver.main.services.walletSessionService
    @Entry var priceAlertService: any GemPriceAlertServiceProtocol = AppResolver.main.services.priceAlertService
    @Entry var deviceService: any GemDeviceServiceProtocol = AppResolver.main.services.deviceService
    @Entry var balanceService: any GemBalanceServiceProtocol = AppResolver.main.services.balanceService
    @Entry var bannerService: any GemBannerServiceProtocol = AppResolver.main.services.bannerService
    @Entry var transactionsService: any GemTransactionsServiceProtocol = AppResolver.main.services.transactionsService
    @Entry var assetsService: any GemAssetsServiceProtocol = AppResolver.main.services.assetsService
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationHandler: NavigationHandler = AppResolver.main.services.navigationHandler
    @Entry var stakeService: any GemStakeServiceProtocol = AppResolver.main.services.stakeService
    @Entry var explorerService: any GemExplorerServiceProtocol = AppResolver.main.services.explorerService
    @Entry var gatewayService: GatewayService = AppResolver.main.services.gatewayService
    @Entry var walletConnector: WalletConnectorService = AppResolver.main.services.walletConnector
    @Entry var connectionStatus: ConnectionStatusObserver = AppResolver.main.services.connectionStatusObserver
    @Entry var walletConnectorPresenter: WalletConnectorPresenter = AppResolver.main.services.walletConnectorPresenter
    @Entry var chainService: GemChainService = AppResolver.main.services.chainService
    @Entry var receiveService: GemReceiveService = AppResolver.main.services.receiveService
    @Entry var transactionFormatter: GemTransactionFormatter = AppResolver.main.services.transactionFormatter
    @Entry var swapService: any GemSwapServiceProtocol = AppResolver.main.services.swapService
    @Entry var nftService: any GemNftServiceProtocol = AppResolver.main.services.nftService
    @Entry var avatarService: any GemAvatarServiceProtocol = AppResolver.main.services.avatarService
    @Entry var appUpdateService: any GemAppUpdateServiceProtocol = AppResolver.main.services.appUpdateService
    @Entry var perpetualService: any GemPerpetualServiceProtocol = AppResolver.main.services.perpetualService
    @Entry var hyperliquidObserverService: any PerpetualObservable = AppResolver.main.services.hyperliquidObserverService
    @Entry var nameService: any GemNameServiceProtocol = AppResolver.main.services.nameService
    @Entry var recentAssetsService: any GemRecentActivityServiceProtocol = AppResolver.main.services.recentAssetsService
    @Entry var addressService: any GemAddressServiceProtocol = AppResolver.main.services.addressService
    @Entry var applicationMetadataService: GemApplicationMetadataService = AppResolver.main.services.viewModelFactory.applicationMetadataService
    @Entry var deeplinkService: GemDeeplinkService = AppResolver.main.services.viewModelFactory.deeplinkService
    @Entry var assetConfig: GemAssetConfigService = AppResolver.main.services.viewModelFactory.assetConfig
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
    @Entry var rewardsService: any GemRewardsServiceProtocol = AppResolver.main.services.rewardsService
    @Entry var searchService: any GemSearchServiceProtocol = AppResolver.main.services.searchService
    @Entry var inAppNotificationService: any GemNotificationServiceProtocol = AppResolver.main.services.inAppNotificationService
    @Entry var portfolioService: any GemPortfolioServiceProtocol = AppResolver.main.services.portfolioService
    @Entry var supportService: any GemSupportServiceProtocol = AppResolver.main.services.supportService
}
