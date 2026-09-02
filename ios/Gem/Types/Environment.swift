// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import class Gemstone.GemDeeplinkService
import protocol Gemstone.GemSwapServiceProtocol
import class Gemstone.GemDeviceKeyService
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemWalletPreferencesServiceProtocol
import protocol Gemstone.GemSupportServiceProtocol
import class Gemstone.GemStreamSubscriptionService
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemBannerServiceProtocol
import protocol Gemstone.GemBalanceServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Preferences
import protocol Gemstone.GemPriceServiceProtocol
import GRDB
import Primitives
import Store
import StreamService
import SwiftUI
import WalletConnector

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var priceService: any GemPriceServiceProtocol = AppResolver.main.services.priceService
    @Entry var streamSubscriptionService: GemStreamSubscriptionService = AppResolver.main.services.streamSubscriptionService
    @Entry var walletPreferencesService: any GemWalletPreferencesServiceProtocol = AppResolver.main.services.walletPreferencesService
    @Entry var observablePreferences: ObservablePreferences = AppResolver.main.services.observablePreferences
    @Entry var preferencesService: any GemPreferencesServiceProtocol = AppResolver.main.services.preferencesService
    @Entry var deviceKeyService: GemDeviceKeyService = AppResolver.main.services.deviceKeyService
    @Entry var walletSessionService: any GemWalletSessionServiceProtocol = AppResolver.main.services.walletSessionService
    @Entry var deviceService: any GemDeviceServiceProtocol = AppResolver.main.services.deviceService
    @Entry var balanceService: any GemBalanceServiceProtocol = AppResolver.main.services.balanceService
    @Entry var bannerService: any GemBannerServiceProtocol = AppResolver.main.services.bannerService
    @Entry var assetsService: any GemAssetsServiceProtocol = AppResolver.main.services.assetsService
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationHandler: NavigationHandler = AppResolver.main.services.navigationHandler
    @Entry var walletConnector: WalletConnectorService = AppResolver.main.services.walletConnector
    @Entry var connectionStatus: ConnectionStatusObserver = AppResolver.main.services.connectionStatusObserver
    @Entry var walletConnectorPresenter: WalletConnectorPresenter = AppResolver.main.services.walletConnectorPresenter
    @Entry var swapService: any GemSwapServiceProtocol = AppResolver.main.services.swapService
    @Entry var nftService: any GemNftServiceProtocol = AppResolver.main.services.nftService
    @Entry var perpetualService: any GemPerpetualServiceProtocol = AppResolver.main.services.perpetualService
    @Entry var hyperliquidObserverService: any PerpetualObservable = AppResolver.main.services.hyperliquidObserverService
    @Entry var recentAssetsService: any GemRecentActivityServiceProtocol = AppResolver.main.services.recentAssetsService
    @Entry var deeplinkService: GemDeeplinkService = AppResolver.main.services.viewModelFactory.deeplinkService
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
    @Entry var supportService: any GemSupportServiceProtocol = AppResolver.main.services.supportService
}
