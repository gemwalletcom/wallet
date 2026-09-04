// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ConnectionStoreTests {
    @Test
    func getConnectionReturnsBoundWallet() throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionStore(db: db)

        let walletA = Wallet.mock(id: .multicoin(address: "0xa"), accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: .multicoin(address: "0xb"), accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(walletA)
        try walletStore.addWallet(walletB)

        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-a"), wallet: walletA))
        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-b"), wallet: walletB))

        #expect(try connectionsStore.getConnection(sessionId: "session-a")?.wallet.id == walletA.id)
        #expect(try connectionsStore.getConnection(sessionId: "session-b")?.wallet.id == walletB.id)
    }

    @Test
    func getConnectionIsNilForNonexistentSession() throws {
        #expect(try ConnectionStore.mock().getConnection(sessionId: "nonexistent") == nil)
    }
}
