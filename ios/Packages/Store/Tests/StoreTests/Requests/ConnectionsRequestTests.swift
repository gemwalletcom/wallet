// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ConnectionsRequestTests {
    @Test
    func returnsConnectionsWithWalletAccounts() throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionStore(db: db)

        let wallet = Wallet.mock(id: .multicoin(address: "0xa"), accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)
        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-a"), wallet: wallet))

        try db.dbQueue.read { db in
            let connections = try ConnectionsRequest().fetch(db)

            #expect(connections.map(\.session.sessionId) == ["session-a"])
            #expect(connections.first?.wallet.accounts.map(\.chain) == [.ethereum])
        }
    }
}
