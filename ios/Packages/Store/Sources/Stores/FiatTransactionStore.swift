// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct FiatTransactionStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    public func setTransactions(walletId: WalletId, transactions: [FiatTransactionData]) throws {
        try db.write { db in
            try FiatTransactionRecord
                .filter(FiatTransactionRecord.Columns.walletId == walletId.id)
                .filter(!transactions.map(\.transaction.id).contains(FiatTransactionRecord.Columns.id))
                .deleteAll(db)

            for transaction in transactions {
                try transaction.record(walletId: walletId.id).upsert(db)
            }
        }
    }
}
