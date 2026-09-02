// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemAppStartServiceProtocol
import GemstoneServices
import AppService
import WalletConnectorService
import ConnectionStatusService
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import Preferences
import protocol Gemstone.GemDeviceServiceProtocol
import Primitives
import PrimitivesComponents
import StreamService
import WalletConnector

extension AppResolver {
    struct Services {
        // Environment-level services
        let walletConnector: WalletConnectorService
        let connectionStatusObserver: ConnectionStatusObserver
        let deviceService: any GemDeviceServiceProtocol
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let streamObserverService: StreamObserverService
        let transactionStateService: any GemTransactionStateServiceProtocol
        let observablePreferences: ObservablePreferences
        let walletSessionService: any GemWalletSessionServiceProtocol
        let appUpdateService: any GemAppUpdateServiceProtocol
        let rateService: RateService
        let onstartService: OnstartService
        let appStartService: any GemAppStartServiceProtocol
        let pushNotificationEnablerService: PushNotificationEnablerService
        let walletConnectorPresenter: WalletConnectorPresenter
        let toastPresenter: ToastPresenter
        let viewModelFactory: ViewModelFactory
        let appLifecycleService: AppLifecycleService
    }
}
