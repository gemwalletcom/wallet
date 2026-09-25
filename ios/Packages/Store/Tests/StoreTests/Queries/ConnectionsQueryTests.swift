// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ConnectionsQueryTests {
    @Test
    func returnsConnectionsWithWalletAccounts() throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0xa"), accounts: [.mock(chain: .ethereum)])
        let db = try DB.mockWithWallets([wallet])
        let connectionsStore = ConnectionStore(db: db)
        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-a"), wallet: wallet))

        try db.dbQueue.read { db in
            let connections = try ConnectionsQuery().fetch(db)

            #expect(connections.map(\.session.sessionId) == ["session-a"])
            #expect(connections.first?.wallet.accounts.map(\.chain) == [.ethereum])
        }
    }
}
