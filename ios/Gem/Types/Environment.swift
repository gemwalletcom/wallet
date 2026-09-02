// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecentActivityServiceProtocol
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemNftServiceProtocol
import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Preferences
import GRDB
import Primitives
import Store
import StreamService
import SwiftUI
import WalletConnector

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var observablePreferences: ObservablePreferences = AppResolver.main.services.observablePreferences
    @Entry var walletSessionService: any GemWalletSessionServiceProtocol = AppResolver.main.services.walletSessionService
    @Entry var deviceService: any GemDeviceServiceProtocol = AppResolver.main.services.deviceService
    @Entry var assetsService: any GemAssetsServiceProtocol = AppResolver.main.services.assetsService
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationHandler: NavigationHandler = AppResolver.main.services.navigationHandler
    @Entry var walletConnector: WalletConnectorService = AppResolver.main.services.walletConnector
    @Entry var connectionStatus: ConnectionStatusObserver = AppResolver.main.services.connectionStatusObserver
    @Entry var walletConnectorPresenter: WalletConnectorPresenter = AppResolver.main.services.walletConnectorPresenter
    @Entry var nftService: any GemNftServiceProtocol = AppResolver.main.services.nftService
    @Entry var recentAssetsService: any GemRecentActivityServiceProtocol = AppResolver.main.services.recentAssetsService
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
}
