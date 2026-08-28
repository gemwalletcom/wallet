// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import protocol Gemstone.GemPerpetualServiceProtocol
import GemstonePrimitivesTestKit
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import ConnectionsService
import ConnectionsServiceTestKit
import ConnectionStatusService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation
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
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
            connectionsService: connectionsService,
            connectionStatusObserver: connectionStatusObserver,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualService: perpetualService,
            perpetualObserver: hyperliquidObserverService,
            walletSessionService: walletSessionService,
        )
    }
}
