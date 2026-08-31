// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import class Gemstone.GemConnectionService
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitivesTestKit
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import WalletConnectorService
import WalletConnectorServiceTestKit
import ConnectionStatusService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation
import PreferencesTestKit
import StreamService
import StreamServiceTestKit

public extension AppLifecycleService {
    static func mock(
        walletConnector: any WalletConnectorServiceable = WalletConnectorServiceMock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: []),
        deviceObserverService: DeviceObserverService = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: any GemStreamSubscriptionServiceProtocol = GemStreamSubscriptionServiceMock(),
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualService: perpetualService,
            perpetualObserver: hyperliquidObserverService,
            walletSessionService: walletSessionService,
            transactionStateService: transactionStateService,
        )
    }
}
