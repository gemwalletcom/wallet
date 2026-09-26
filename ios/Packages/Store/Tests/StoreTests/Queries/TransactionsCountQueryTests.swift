// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct TransactionsCountQueryTests {
    @Test
    func countMatchesActivityList() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .smartChain)])], assets: [
            .mock(asset: .mock(), score: .mock(rank: 20)),
            .mock(asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18), score: .mock(rank: 0)),
        ])
        let walletId = WalletId.multicoin(address: "0x0000000000000000000000000000000000000000")
        let store = TransactionStore(db: db)

        try store.addTransactions(walletId: walletId, transactions: [
            .mock(.mock(id: TransactionId(chain: .bitcoin, hash: "1"), state: .pending)),
            .mock(.mock(id: TransactionId(chain: .bitcoin, hash: "2"), state: .inTransit)),
            .mock(.mock(id: TransactionId(chain: .bitcoin, hash: "3"), state: .confirmed)),
            .mock(.mock(id: TransactionId(chain: .smartChain, hash: "4"), assetId: AssetId(chain: .smartChain), state: .pending)),
        ])

        let filter = TransactionsFilter.mock(states: [.pending, .inTransit], assetRankGreaterThan: 15)
        let (count, listed) = try db.dbQueue.read { db in
            try (
                TransactionsCountQuery(walletId: walletId, type: .all, filter: filter).fetch(db),
                TransactionsQuery.fetch(db, type: .all, filter: filter, walletId: walletId),
            )
        }

        #expect(count == 2)
        #expect(listed.map(\.transaction.state).asSet() == [.pending, .inTransit])
        #expect(count == listed.count)
    }
}
