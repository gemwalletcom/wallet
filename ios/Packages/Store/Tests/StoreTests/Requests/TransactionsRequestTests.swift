// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import os
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct TransactionsRequestTests {
    @Test
    func assetScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = AssetId(chain: .ethereum)

        #expect(
            TransactionsRequest.assetScene(walletId: walletId, assetId: assetId, limit: 250) ==
                TransactionsRequest(
                    walletId: walletId,
                    type: .asset(assetId: assetId),
                    limit: 250,
                ),
        )
    }

    @Test
    func perpetualScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = Asset.mockHypercoreUSDC().id

        #expect(
            TransactionsRequest.perpetualScene(
                walletId: walletId,
                assetId: assetId,
                types: [.perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition],
                limit: 250,
            ) ==
                TransactionsRequest(
                    walletId: walletId,
                    type: .asset(assetId: assetId),
                    filters: [.types([
                        TransactionType.perpetualOpenPosition.rawValue,
                        TransactionType.perpetualClosePosition.rawValue,
                        TransactionType.perpetualModifyPosition.rawValue,
                    ])],
                    limit: 250,
                ),
        )
    }

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

        #expect(!observedRegion { _ = try TransactionsRequest(walletId: walletId, type: .all).fetch($0) }.isModified(byEventsOfKind: priceUpdate))
        #expect(observedRegion { _ = try TransactionRequest(walletId: walletId, recordId: 1).fetch($0) }.isModified(byEventsOfKind: priceUpdate))
    }
}
