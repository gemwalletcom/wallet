// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct TransactionsCountRequestTests {
    @Test
    func countMatchesActivityList() throws {
        let db = DB.mockAssets(assets: [
            .mock(asset: .mock(), score: .mock(rank: 20)),
            .mock(asset: .mockBNB(), score: .mock(rank: 0)),
        ])
        let walletId = WalletId.multicoin(address: "0x0000000000000000000000000000000000000000")
        let store = TransactionStore(db: db)

        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: TransactionId(chain: .bitcoin, hash: "1"), state: .pending),
            .mock(transactionId: TransactionId(chain: .bitcoin, hash: "2"), state: .inTransit),
            .mock(transactionId: TransactionId(chain: .bitcoin, hash: "3"), state: .confirmed),
            .mock(transactionId: TransactionId(chain: .smartChain, hash: "4"), state: .pending, assetId: AssetId(chain: .smartChain)),
        ])

        let filters: [TransactionsRequestFilter] = [.assetRankGreaterThan(15)]
        let (count, listed) = try db.dbQueue.read { db in
            try (
                TransactionsCountRequest(walletId: walletId, type: .pending, filters: filters).fetch(db),
                TransactionsRequest.fetch(db, type: .pending, filters: filters, walletId: walletId),
            )
        }

        #expect(count == 2)
        #expect(listed.map(\.transaction.state).asSet() == [.pending, .inTransit])
        #expect(count == listed.count)
    }
}
