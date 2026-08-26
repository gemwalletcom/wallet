// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import ConnectionsService
import ConnectionsServiceTestKit
import ConnectionStatusService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import StreamService
import StreamServiceTestKit
import WalletSessionService
import WalletSessionServiceTestKit

public extension AppLifecycleService {
    static func mock(
        preferences: Preferences = .mock(),
        connectionsService: ConnectionsService = .mock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(monitors: []),
        deviceObserverService: DeviceObserverService = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: StreamSubscriptionService = .mock(),
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
        perpetualService: any PerpetualServiceable = PerpetualServiceMock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
            preferences: preferences,
            connectionsService: connectionsService,
            connectionStatusObserver: connectionStatusObserver,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualEnablerService: PerpetualEnablerService(
                observer: hyperliquidObserverService,
                service: perpetualService,
                preferences: preferences,
            ),
            walletSessionService: walletSessionService,
        )
    }
}
