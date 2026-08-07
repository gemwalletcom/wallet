// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import Store
import StoreTestKit
import Testing

struct TransactionsCountRequestTests {
    @Test
    func countsSelectedStatesAboveRank() throws {
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

        let count = try db.dbQueue.read { db in
            try TransactionsCountRequest(walletId: walletId, states: [.pending, .inTransit], rank: 15).fetch(db)
        }

        #expect(count == 2)
    }
}
