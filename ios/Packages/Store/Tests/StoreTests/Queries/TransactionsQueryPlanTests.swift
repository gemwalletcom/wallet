// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives
@testable import Store
import StoreTestKit
import Testing

struct TransactionsQueryPlanTests {
    @Test
    func walletHistoryIsReadNewestFirstFromTheIndex() throws {
        let plan = try DB.mock().dbQueue.read { db in
            var statements: [String] = []
            db.trace { statements.append($0.expandedDescription) }
            _ = try TransactionsQuery.fetch(db, type: .all, filters: [], walletId: .multicoin(address: "wallet"), limit: 1000)
            db.trace(options: [], nil)
            return try Row.fetchAll(db, sql: "EXPLAIN QUERY PLAN \(statements.last { $0.contains("ORDER BY") } ?? "")").map { $0["detail"] as String }
        }
        #expect(plan.contains { $0.contains("index_\(TransactionRecord.databaseTableName)_on_walletId_date") }, "\(plan)")
        #expect(!plan.contains { $0.contains("TEMP B-TREE FOR ORDER BY") }, "\(plan)")
    }
}
