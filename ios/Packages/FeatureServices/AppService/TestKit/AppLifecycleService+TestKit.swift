// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import class Gemstone.GemConnectionService
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitivesTestKit
import protocol Gemstone.GemDeviceServiceProtocol
import Store
import StoreTestKit
import protocol Gemstone.GemStreamSubscriptionServiceProtocol
import WalletConnectorService
import WalletConnectorServiceTestKit
import ConnectionStatusService
import GemstoneServices
import GemstoneServicesTestKit
import class Gemstone.GemWalletSessionService
import protocol Gemstone.GemWalletSessionServiceProtocol
import Foundation
import PreferencesTestKit
import StreamService
import StreamServiceTestKit

public extension AppLifecycleService {
    static func mock(
        walletConnector: any WalletConnectorServiceable = WalletConnectorServiceMock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: []),
        deviceService: any GemDeviceServiceProtocol = GemDeviceServiceMock(),
        subscriptionsObserver: SubscriptionsObserver = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: any GemStreamSubscriptionServiceProtocol = GemStreamSubscriptionServiceMock(),
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        walletSessionService: any GemWalletSessionServiceProtocol = GemWalletSessionService.mock(),
        transactionStateService: any GemTransactionStateServiceProtocol = GemTransactionStateServiceMock(),
    ) -> AppLifecycleService {
        AppLifecycleService(
            walletConnector: walletConnector,
            connectionStatusObserver: connectionStatusObserver,
            deviceService: deviceService,
            subscriptionsObserver: subscriptionsObserver,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            perpetualService: perpetualService,
            perpetualObserver: hyperliquidObserverService,
            walletSessionService: walletSessionService,
            transactionStateService: transactionStateService,
        )
    }
}
