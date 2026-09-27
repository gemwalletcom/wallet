// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletConnectServiceProtocol
import GemstonePrimitivesTestKit
import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension ConnectionsSceneViewModel {
    @MainActor
    static func mock(
        connector: any WalletConnectorServiceable = WalletConnectorServiceMock(),
        service: any GemWalletConnectServiceProtocol = GemWalletConnectServiceMock(),
    ) -> ConnectionsSceneViewModel {
        ConnectionsSceneViewModel(connector: connector, service: service)
    }
}
