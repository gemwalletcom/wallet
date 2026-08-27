// Copyright (c). Gem Wallet. All rights reserved.

import ConnectionsService
import ConnectionsServiceTestKit
import Preferences
import Testing
@testable import WalletConnector
import WalletConnectorServiceTestKit

struct ConnectionsServiceTests {
    let preferences: Preferences = .mock()

    @Test
    func walletConnectActivation() async throws {
        try await firstRun()
        try await secondRun()
    }

    private func firstRun() async throws {
        let connector = WalletConnectorServiceMock()
        let service: ConnectionsService = .mock(connector: connector, preferences: preferences)

        try await service.setup()
        await #expect(connector.isSetup == false)
        #expect(service.isWalletConnectActivated == false)

        try await service.pair(uri: .empty)
        await #expect(connector.isSetup == true)
        #expect(service.isWalletConnectActivated)
    }

    private func secondRun() async throws {
        let connector = WalletConnectorServiceMock()
        let service: ConnectionsService = .mock(connector: connector, preferences: preferences)

        try await service.setup()
        await #expect(connector.isSetup == true)
        #expect(service.isWalletConnectActivated)
    }
}

extension ConnectionsService {
    static func mock(
        connector: WalletConnectorServiceMock,
        preferences: Preferences,
    ) -> ConnectionsService {
        ConnectionsService(
            connector: connector,
            preferences: preferences,
        )
    }
}
