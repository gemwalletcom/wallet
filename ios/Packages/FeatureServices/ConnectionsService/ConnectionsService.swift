// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnectorService

public final class ConnectionsService: Sendable {
    private let connector: WalletConnectorServiceable
    private let lock = NSLock()
    nonisolated(unsafe) private var isConnectorReady = false

    public init(connector: WalletConnectorServiceable) {
        self.connector = connector
    }
}

public extension ConnectionsService {
    func setup() async throws {
        try connector.configure()
        if try await connector.hasSessions() {
            await setupConnector()
        }
    }

    func pair(uri: String) async throws {
        await setupConnector()
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
    private func setupConnector() async {
        let alreadyReady = lock.withLock {
            defer { isConnectorReady = true }
            return isConnectorReady
        }
        if alreadyReady {
            return
        }
        await connector.setup()
    }
}
