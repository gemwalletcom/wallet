// Copyright (c). Gem Wallet. All rights reserved.

@testable import ConnectionsService
import Foundation
import WalletConnectorService
import WalletConnectorServiceTestKit

public extension ConnectionsService {
    static func mock(connector: WalletConnectorServiceable = WalletConnectorServiceMock()) -> ConnectionsService {
        ConnectionsService(connector: connector)
    }
}
