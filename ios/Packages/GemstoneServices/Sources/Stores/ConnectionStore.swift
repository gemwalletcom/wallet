// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConnectionStore
import struct Gemstone.WalletConnection
import struct Gemstone.WalletConnectionSession
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneConnectionStore: GemConnectionStore, @unchecked Sendable {
    private let store: ConnectionStore

    public init(store: ConnectionStore) {
        self.store = store
    }

    public func getConnection(sessionId: String) async throws -> Gemstone.WalletConnection? {
        try store.getConnection(sessionId: sessionId).map { $0.map() }
    }

    public func getSessions() async throws -> [Gemstone.WalletConnectionSession] {
        try store.getSessions().map { $0.map() }
    }

    public func addConnection(connection: Gemstone.WalletConnection) async throws {
        try store.addConnection(connection.map())
    }

    public func updateSession(session: Gemstone.WalletConnectionSession) async throws {
        try store.updateConnectionSession(session.map())
    }

    public func deleteSessions(sessionIds: [String]) async throws {
        _ = try store.delete(ids: sessionIds)
    }
}
