// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives
import Store
import WalletConnectorService

public final class ConnectionsService: Sendable {
    private let store: ConnectionsStore
    private let signer: any WalletConnectorSignable
    private let connector: WalletConnectorServiceable
    private let preferences: Preferences

    public var isWalletConnectActivated: Bool {
        get { preferences.isWalletConnectActivated == true }
        set { preferences.isWalletConnectActivated = newValue }
    }

    public init(
        store: ConnectionsStore,
        signer: any WalletConnectorSignable,
        connector: WalletConnectorServiceable,
        preferences: Preferences = .standard,
    ) {
        self.store = store
        self.signer = signer
        self.connector = connector
        self.preferences = preferences
    }

    public convenience init(
        store: ConnectionsStore,
        signer: any WalletConnectorSignable,
        nodeProvider: any NodeURLFetchable,
        preferences: Preferences = .standard,
    ) {
        self.init(
            store: store,
            signer: signer,
            connector: WalletConnectorService(signer: signer, nodeProvider: nodeProvider),
            preferences: preferences,
        )
    }
}

// MARK: - Public

public extension ConnectionsService {
    func setup() async throws {
        checkExistSessions()
        try connector.configure()
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
        try await disconnect(sessionId: session.sessionId)
    }

    func updateSessions() {
        connector.updateSessions()
    }
}

// MARK: - Private

extension ConnectionsService {
    private func disconnect(sessionId: String) async throws {
        _ = try store.delete(ids: [sessionId])
        try await connector.disconnect(sessionId: sessionId)
    }

    private func setupConnector() async throws {
        if !isWalletConnectActivated {
            isWalletConnectActivated = true
        }
        await connector.setup()
    }

    // TODO: - Remove migration 08.2025
    private func checkExistSessions() {
        if preferences.isWalletConnectActivated == nil {
            isWalletConnectActivated = (try? store.getSessions().isNotEmpty) == true
        }
    }
}
