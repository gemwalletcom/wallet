// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Observation
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
        #expect(observedRegion { _ = try TransactionQuery(walletId: walletId, key: .record(1)).fetch($0) }.isModified(byEventsOfKind: priceUpdate))
    }

    @Test(.timeLimit(.minutes(1))) @MainActor
    func aLargerLimitKeepsTheShownRowsUntilItsRowsArrive() async throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])], assets: [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))])
        let older = TransactionId(chain: .ethereum, hash: "older")
        let newer = TransactionId(chain: .ethereum, hash: "newer")
        try TransactionStore(db: db).addTransactions(walletId: .mock(), transactions: [
            .mock(.mock(id: older, assetId: Chain.ethereum.assetId, feeAssetId: Chain.ethereum.assetId, createdAt: Date(timeIntervalSince1970: 1))),
            .mock(.mock(id: newer, assetId: Chain.ethereum.assetId, feeAssetId: Chain.ethereum.assetId, createdAt: Date(timeIntervalSince1970: 2))),
        ])
        let query = ObservableQuery(TransactionsQuery(walletId: .mock(), type: .all, limit: 1), initialValue: [])
        query.bind(dbQueue: db.dbQueue)
        #expect(query.value.map(\.transaction.id) == [newer])

        query.setLimit(2)
        #expect(query.value.map(\.transaction.id) == [newer])

        while query.value.count < 2 {
            await withCheckedContinuation { changed in
                withObservationTracking { _ = query.value } onChange: { changed.resume() }
            }
        }
        #expect(query.value.map(\.transaction.id) == [newer, older])
    }

    @Test @MainActor
    func aCreatedFromDateKeepsTransactionsFromThatTimeOn() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])], assets: [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))])
        let older = TransactionId(chain: .ethereum, hash: "older")
        let newer = TransactionId(chain: .ethereum, hash: "newer")
        try TransactionStore(db: db).addTransactions(walletId: .mock(), transactions: [
            .mock(.mock(id: older, assetId: Chain.ethereum.assetId, feeAssetId: Chain.ethereum.assetId, createdAt: Date(timeIntervalSince1970: 1))),
            .mock(.mock(id: newer, assetId: Chain.ethereum.assetId, feeAssetId: Chain.ethereum.assetId, createdAt: Date(timeIntervalSince1970: 2))),
        ])
        let query = TransactionsQuery(walletId: .mock(), type: .all, filter: .mock(createdFrom: Date(timeIntervalSince1970: 2)))

        #expect(try db.dbQueue.read { try query.fetch($0) }.map(\.transaction.id) == [newer])
    }
}
