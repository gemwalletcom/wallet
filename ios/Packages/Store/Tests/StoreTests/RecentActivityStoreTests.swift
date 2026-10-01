// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct RecentActivityStoreTests {
    @Test
    func openingAnAssetAgainKeepsOneRowPerAssetWalletAndType() throws {
        let wallet = Wallet.mock()
        let db = DB.mock(chains: [.bitcoin, .ethereum], wallets: [wallet])
        let store = RecentActivityStore(db: db)
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: wallet.id, type: .search, createdAt: Date(timeIntervalSince1970: 1))
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: .mock(chain: .ethereum), walletId: wallet.id, type: .search, createdAt: Date(timeIntervalSince1970: 2))
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: wallet.id, type: .perpetual)

        let rows = try db.dbQueue.read { try RecentActivityRecord.order(RecentActivityRecord.Columns.createdAt).fetchAll($0) }
        #expect(rows.count == 2)
        #expect(rows.first?.toAssetId == .mock(chain: .ethereum), "the latest open replaces the earlier one for the same asset, wallet and type")
    }

    @Test
    func clearWithoutAWalletForgetsTheTypeInEveryWallet() throws {
        let wallet = Wallet.mock()
        let otherWallet = Wallet.mock(id: .mock(address: "0x2"))
        let store = RecentActivityStore.mock(db: .mock(chains: [.bitcoin, .ethereum], wallets: [wallet, otherWallet]))
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: wallet.id, type: .perpetual)
        try store.add(assetId: .mock(chain: .ethereum), toAssetId: nil, walletId: wallet.id, type: .search)
        try store.add(assetId: .mock(chain: .bitcoin), toAssetId: nil, walletId: otherWallet.id, type: .perpetual)

        try store.clear(scope: .allWallets, types: [.perpetual])

        #expect(try store.getRecent(walletId: wallet.id, types: RecentActivityType.allCases, limit: 10).map(\.asset.id) == [.mock(chain: .ethereum)])
        #expect(try store.getRecent(walletId: otherWallet.id, types: RecentActivityType.allCases, limit: 10).isEmpty)
    }
}
