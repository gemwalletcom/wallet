// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemWalletConnectServiceProtocol
import GemstonePrimitivesTestKit
import WalletConnector
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension ConnectionsViewModel {
    @MainActor
    static func mock(
        connector: any WalletConnectorServiceable = WalletConnectorServiceMock(),
        service: any GemWalletConnectServiceProtocol = GemWalletConnectServiceMock(),
    ) -> ConnectionsViewModel {
        ConnectionsViewModel(connector: connector, service: service)
    }
}
