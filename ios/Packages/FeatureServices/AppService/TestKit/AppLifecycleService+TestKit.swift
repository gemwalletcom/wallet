// Copyright (c). Gem Wallet. All rights reserved.

@testable import AppService
import ConnectionStatusService
import Foundation
import class Gemstone.GemConnectionService
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemTransactionStateServiceProtocol
import class Gemstone.GemWalletSessionService
import protocol Gemstone.GemWalletSessionServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Store
import StoreTestKit
import StreamService
import StreamServiceTestKit
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension AppLifecycleService {
    static func mock(
        walletConnector: any WalletConnectorServiceable = WalletConnectorServiceMock(),
        connectionStatusObserver: ConnectionStatusObserver = ConnectionStatusObserver(connectionService: GemConnectionService(), monitors: []),
        deviceService: any GemDeviceServiceProtocol = GemDeviceServiceMock(),
        subscriptionsObserver: SubscriptionsObserver = .mock(),
        streamObserverService: StreamObserverService = .mock(),
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
            perpetualService: perpetualService,
            perpetualObserver: hyperliquidObserverService,
            walletSessionService: walletSessionService,
            transactionStateService: transactionStateService,
        )
    }
}
