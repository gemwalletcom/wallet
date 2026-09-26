// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
@testable import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ConnectionStoreTests {
    @Test
    func connectionsBindToTheirWallets() async throws {
        let walletA = Wallet.mock(id: .multicoin(address: "0xa"), name: "Wallet A", accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: .multicoin(address: "0xb"), name: "Wallet B", accounts: [.mock(chain: .ethereum)])
        let store = GemstoneConnectionStore(store: .mock(db: .mock(wallets: [walletA, walletB])))

        try await store.addConnection(connection: WalletConnection.mock(session: .mock(id: "a", sessionId: "a"), wallet: walletA).toGem())
        try await store.addConnection(connection: WalletConnection.mock(session: .mock(id: "b", sessionId: "b"), wallet: walletB).toGem())

        let connectionA = try #require(try await store.getConnection(sessionId: "a").map { $0.toPrimitives() })
        let connectionB = try #require(try await store.getConnection(sessionId: "b").map { $0.toPrimitives() })
        #expect(connectionA.wallet.id == walletA.id)
        #expect(connectionB.wallet.id == walletB.id)
        #expect(connectionA.wallet.accounts.map(\.chain) == [.ethereum])
        #expect(try await store.getConnection(sessionId: "missing") == nil)
    }

    @Test
    func updatesAndDeletesSessions() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xa"), accounts: [.mock(chain: .ethereum)])
        let store = GemstoneConnectionStore(store: .mock(db: .mock(wallets: [wallet])))
        try await store.addConnection(connection: WalletConnection.mock(session: .mock(id: "a", sessionId: "a", chains: [.ethereum]), wallet: wallet).toGem())

        try await store.updateSession(session: WalletConnectionSession.mock(id: "a", sessionId: "a", chains: [.ethereum, .solana]).toGem())
        let sessions = try await store.getSessions().map { $0.toPrimitives() }
        #expect(sessions.map(\.chains) == [[.ethereum, .solana]])

        try await store.deleteSessions(sessionIds: ["a"])
        #expect(try await store.getSessions().isEmpty)
    }
}
