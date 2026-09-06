// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct FiatTransactionStoreTests {
    @Test
    func setTransactionsReplacesWalletSnapshot() throws {
        let db = DB.mockAssets()
        let store = FiatTransactionStore(db: db)

        try store.setTransactions(walletId: .mock(), transactions: [
            .mock(transaction: .mock(id: "2", status: .complete)),
            .mock(transaction: .mock(id: "1", status: .failed)),
        ])

        #expect(try storedIds(db) == ["1", "2"])

        try store.setTransactions(walletId: .mock(), transactions: [
            .mock(transaction: .mock(id: "2", status: .complete)),
        ])

        #expect(try storedIds(db) == ["2"])
    }

    private func storedIds(_ db: DB) throws -> [String] {
        try db.dbQueue
            .read { try FiatTransactionsRequest(walletId: .mock()).fetch($0) }
            .map(\.transaction.id)
            .sorted()
    }
}
