// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConnectionStore
import typealias Gemstone.WalletConnection
import typealias Gemstone.WalletConnectionSession
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneConnectionStore: GemConnectionStore, @unchecked Sendable {
    private let store: ConnectionsStore

    public init(store: ConnectionsStore) {
        self.store = store
    }

    public func getConnection(sessionId: String) async throws -> Gemstone.WalletConnection? {
        try store.getConnection(sessionId: sessionId).map { try $0.json() }
    }

    public func getSessions() async throws -> [Gemstone.WalletConnectionSession] {
        try store.getSessions().map { try $0.json() }
    }

    public func addConnection(connection: Gemstone.WalletConnection) async throws {
        try store.addConnection(Primitives.WalletConnection(connection))
    }

    public func updateSession(session: Gemstone.WalletConnectionSession) async throws {
        try store.updateConnectionSession(Primitives.WalletConnectionSession(session))
    }

    public func deleteSessions(sessionIds: [String]) async throws {
        _ = try store.delete(ids: sessionIds)
    }
}
