// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives
import WalletConnectorService

public final class ConnectionsService: Sendable {
    private let connector: WalletConnectorServiceable
    private let preferences: Preferences

    public var isWalletConnectActivated: Bool {
        get { preferences.isWalletConnectActivated == true }
        set { preferences.isWalletConnectActivated = newValue }
    }

    public init(
        connector: WalletConnectorServiceable,
        preferences: Preferences = .standard,
    ) {
        self.connector = connector
        self.preferences = preferences
    }
}

public extension ConnectionsService {
    func setup() async throws {
        try connector.configure()
        if preferences.isWalletConnectActivated == nil {
            isWalletConnectActivated = connector.hasSessions
        }
        if isWalletConnectActivated {
            try await setupConnector()
        }
    }

    func pair(uri: String) async throws {
        if !isWalletConnectActivated {
            try await setupConnector()
        }
        try await connector.pair(uri: uri)
    }

    func disconnect(session: WalletConnectionSession) async throws {
        try await connector.disconnect(sessionId: session.sessionId)
    }

    func updateSessions() {
        connector.updateSessions()
    }
}

extension ConnectionsService {
    private func setupConnector() async throws {
        if !isWalletConnectActivated {
            isWalletConnectActivated = true
        }
        await connector.setup()
    }
}
