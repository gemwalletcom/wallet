// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import GRDB
import Primitives
import SwiftUI
import WalletConnector
import WalletConnectorService

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var observablePreferences: ObservablePreferences = AppResolver.main.services.observablePreferences
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var navigationRouter: NavigationRouter = AppResolver.main.services.navigationRouter
    @Entry var walletConnector: WalletConnectorService = AppResolver.main.services.walletConnector
    @Entry var walletConnectorPresenter: WalletConnectorPresenter = AppResolver.main.services.walletConnectorPresenter
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
}
