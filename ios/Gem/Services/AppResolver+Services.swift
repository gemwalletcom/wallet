// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemRecentActivityService
import protocol Gemstone.GemTransactionStateServiceProtocol
import class Gemstone.GemChainService
import class Gemstone.GemDeviceKeyService
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemWalletPreferencesServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemAvatarServiceProtocol
import class Gemstone.GemStreamSubscriptionService
import protocol Gemstone.GemPriceAlertServiceProtocol
import protocol Gemstone.GemAppStartServiceProtocol
import protocol Gemstone.GemPortfolioServiceProtocol
import protocol Gemstone.GemSearchServiceProtocol
import protocol Gemstone.GemSwapServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Preferences
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import protocol Gemstone.GemNotificationServiceProtocol
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
        let walletConnector: WalletConnectorService
        let connectionStatusObserver: ConnectionStatusObserver
        let deviceService: any GemDeviceServiceProtocol
        let serviceStatusService: any GemServiceStatusProtocol
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let priceAlertService: any GemPriceAlertServiceProtocol
        let streamObserverService: StreamObserverService
        let streamSubscriptionService: GemStreamSubscriptionService
        let priceService: any GemPriceServiceProtocol
        let stakeService: any GemStakeServiceProtocol
        let transactionStateService: any GemTransactionStateServiceProtocol
        let walletPreferencesService: any GemWalletPreferencesServiceProtocol
        let preferencesService: any GemPreferencesServiceProtocol
        let deviceKeyService: GemDeviceKeyService
        let observablePreferences: ObservablePreferences
        let walletSessionService: any GemWalletSessionServiceProtocol
        let assetsService: any GemAssetsServiceProtocol
        let explorerService: any GemExplorerServiceProtocol
        let nftService: any GemNftServiceProtocol
        let avatarService: any GemAvatarServiceProtocol
        let swapService: any GemSwapServiceProtocol
        let appUpdateService: any GemAppUpdateServiceProtocol
        let rateService: RateService
        let onstartService: OnstartService
        let appStartService: any GemAppStartServiceProtocol
        let pushNotificationEnablerService: PushNotificationEnablerService
        let walletConnectorPresenter: WalletConnectorPresenter
        let chainService: GemChainService
        let perpetualService: any GemPerpetualServiceProtocol
        let hyperliquidObserverService: any PerpetualObservable
        let recentAssetsService: any GemRecentActivityServiceProtocol
        let toastPresenter: ToastPresenter
        let viewModelFactory: ViewModelFactory
        let searchService: any GemSearchServiceProtocol
        let appLifecycleService: AppLifecycleService
        let inAppNotificationService: any GemNotificationServiceProtocol
        let portfolioService: any GemPortfolioServiceProtocol
        let supportService: any GemSupportServiceProtocol
    }
}
