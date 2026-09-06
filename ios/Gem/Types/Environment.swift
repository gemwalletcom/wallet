// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import Foundation
import GRDB
import Primitives
import Store
import StreamService
import SwiftUI
import WalletConnector

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var observablePreferences: ObservablePreferences = AppResolver.main.services.observablePreferences
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationHandler: NavigationHandler = AppResolver.main.services.navigationHandler
    @Entry var walletConnector: WalletConnectorService = AppResolver.main.services.walletConnector
    @Entry var connectionStatus: ConnectionStatusObserver = AppResolver.main.services.connectionStatusObserver
    @Entry var walletConnectorPresenter: WalletConnectorPresenter = AppResolver.main.services.walletConnectorPresenter
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
}
