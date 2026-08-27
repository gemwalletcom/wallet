// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionsService
import ConnectionsServiceTestKit
import Testing
@testable import WalletConnector
import WalletConnectorServiceTestKit

struct ConnectionsServiceTests {
    @Test
    func setupSkipsConnectorWithoutSessions() async throws {
        let connector = WalletConnectorServiceMock(hasSessions: false)
        let service: ConnectionsService = .mock(connector: connector)

        try await service.setup()
        await #expect(connector.isSetup == false)

        try await service.pair(uri: .empty)
        await #expect(connector.isSetup == true)
    }

    @Test
    func setupStartsConnectorWithStoredSessions() async throws {
        let connector = WalletConnectorServiceMock(hasSessions: true)
        let service: ConnectionsService = .mock(connector: connector)

        try await service.setup()
        await #expect(connector.isSetup == true)
    }
}
