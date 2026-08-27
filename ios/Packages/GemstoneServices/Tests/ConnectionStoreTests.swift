// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import GemstoneServices

struct ConnectionStoreTests {
    @Test
    func connectionsBindToTheirWallets() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let walletA = Wallet.mock(id: .multicoin(address: "0xa"), name: "Wallet A", accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: .multicoin(address: "0xb"), name: "Wallet B", accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(walletA)
        try walletStore.addWallet(walletB)
        let store = GemstoneConnectionStore(store: ConnectionsStore(db: db))

        try await store.addConnection(connection: WalletConnection(session: .mock(id: "a", sessionId: "a"), wallet: walletA).json())
        try await store.addConnection(connection: WalletConnection(session: .mock(id: "b", sessionId: "b"), wallet: walletB).json())

        let connectionA = try #require(try await store.getConnection(sessionId: "a").map { try WalletConnection($0) })
        let connectionB = try #require(try await store.getConnection(sessionId: "b").map { try WalletConnection($0) })
        #expect(connectionA.wallet.id == walletA.id)
        #expect(connectionB.wallet.id == walletB.id)
        #expect(connectionA.wallet.accounts.map(\.chain) == [.ethereum])
        #expect(try await store.getConnection(sessionId: "missing") == nil)
    }

    @Test
    func updatesAndDeletesSessions() async throws {
        let db = DB.mockWithChains([.ethereum])
        let wallet = Wallet.mock(id: .multicoin(address: "0xa"), accounts: [.mock(chain: .ethereum)])
        try WalletStore(db: db).addWallet(wallet)
        let store = GemstoneConnectionStore(store: ConnectionsStore(db: db))
        try await store.addConnection(connection: WalletConnection(session: .mock(id: "a", sessionId: "a", chains: [.ethereum]), wallet: wallet).json())

        try await store.updateSession(session: WalletConnectionSession.mock(id: "a", sessionId: "a", chains: [.ethereum, .solana]).json())
        let sessions = try await store.getSessions().map { try WalletConnectionSession($0) }
        #expect(sessions.map(\.chains) == [[.ethereum, .solana]])

        try await store.deleteSessions(sessionIds: ["a"])
        #expect(try await store.getSessions().isEmpty)
    }
}
