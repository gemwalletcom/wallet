// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import ConnectionStatusService
import Foundation
import protocol Gemstone.GemAppStartServiceProtocol
import protocol Gemstone.GemAppUpdateServiceProtocol
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemWalletSessionServiceProtocol
import GemstoneServices
import Primitives
import PrimitivesComponents
import StreamService
import WalletConnector
import WalletConnectorService

extension AppResolver {
    struct Services {
        // Environment-level services
        let walletConnector: WalletConnectorService
        let connectionStatusObserver: ConnectionStatusObserver
        let devicePlatform: GemstoneDevicePlatform
        let deviceService: any GemDeviceServiceProtocol
        let navigationRouter: NavigationRouter
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
