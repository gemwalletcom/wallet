// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import GemstonePrimitivesTestKit
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import ConnectionsService
import ConnectionsServiceTestKit
import ConnectionStatusService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation
import Preferences
import PreferencesTestKit
import StreamService
import StreamServiceTestKit

public extension AppLifecycleService {
    static func mock(
        connectionsService: ConnectionsService = .mock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(monitors: []),
        deviceObserverService: DeviceObserverService = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: any GemStreamSubscriptionServiceProtocol = GemStreamSubscriptionServiceMock(),
        preferences: Preferences = .standard,
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
        perpetualService: any PerpetualServiceable = PerpetualServiceMock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
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
