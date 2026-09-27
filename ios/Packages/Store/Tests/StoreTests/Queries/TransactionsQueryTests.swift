// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import os
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct TransactionsQueryTests {
    @Test @MainActor
    func listIgnoresPriceUpdatesWhileTheDetailFollowsThem() {
        let walletId = WalletId.multicoin(address: "wallet")
        let priceUpdate = DatabaseEventKind.update(tableName: PriceRecord.databaseTableName, columnNames: [PriceRecord.Columns.price.name])
        let observedRegion = { (fetch: @escaping @Sendable (Database) throws -> Void) in
            let region = OSAllocatedUnfairLock(initialState: DatabaseRegion())
            let cancellable = ValueObservation.tracking { db in try? fetch(db) }
                .handleEvents(willTrackRegion: { tracked in region.withLock { $0 = tracked } })
                .start(in: DB.mock().dbQueue, scheduling: .immediate, onError: { _ in }, onChange: { _ in })
            cancellable.cancel()
            return region.withLock { $0 }
        }

        #expect(!observedRegion { _ = try TransactionsQuery(walletId: walletId, type: .all).fetch($0) }.isModified(byEventsOfKind: priceUpdate))
        #expect(observedRegion { _ = try TransactionQuery(walletId: walletId, recordId: 1).fetch($0) }.isModified(byEventsOfKind: priceUpdate))
    }
}
