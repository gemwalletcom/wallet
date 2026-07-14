// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import ConnectionsService
import ConnectionsServiceTestKit
import ConnectivityService
import ConnectivityServiceTestKit
import DeviceService
import ConnectionStatusService
import DeviceServiceTestKit
import Foundation
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import StreamService
import StreamServiceTestKit

public extension AppLifecycleService {
    static func mock(
        preferences: Preferences = .mock(),
        connectionsService: ConnectionsService = .mock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(connectivity: ConnectivityService.mock()),
        deviceObserverService: DeviceObserverService = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: StreamSubscriptionService = .mock(),
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
            preferences: preferences,
            connectionsService: connectionsService,
            connectionStatusObserver: connectionStatusObserver,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            hyperliquidObserverService: hyperliquidObserverService,
        )
    }
}
