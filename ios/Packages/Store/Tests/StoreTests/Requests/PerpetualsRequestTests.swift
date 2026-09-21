// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PerpetualsRequestTests {
    @Test
    func browsingListsTradedMarketsAndKeepsAPinnedOneOutsideTheCap() throws {
        let db = DB.mockAssets(assets: [.mock(asset: .mockEthereum()), .mock(asset: .mock(id: .mock(.bitcoin), symbol: "BTC"))])
        let store = PerpetualStore(db: db)
        let traded = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "ETH"), assetId: .mock(.ethereum), volume24h: 10)
        let delisted = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "BTC"), name: "BTC-USD", assetId: .mock(.bitcoin), volume24h: 0)

        try store.upsertPerpetuals([traded, delisted])

        try db.dbQueue.read { db in
            let markets = try PerpetualsRequest(searchQuery: "", limit: 1, requiresVolume: true).fetch(db)
            #expect(markets.map(\.perpetual.id) == [traded.id], "a market nobody trades is not offered for discovery")
        }

        try store.setPinned(for: [delisted.id.identifier], value: true)

        try db.dbQueue.read { db in
            let markets = try PerpetualsRequest(searchQuery: "", limit: 1, requiresVolume: true).fetch(db)
            #expect(markets.map(\.perpetual.id) == [delisted.id], "a pinned market is read before the cap cuts the list")
        }
    }

    @Test
    func aSearchReachesAMarketThatHasNotTraded() throws {
        let db = DB.mockAssets(assets: [.mock(asset: .mock(id: .mock(.bitcoin), symbol: "BTC"))])
        let store = PerpetualStore(db: db)
        let delisted = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "BTC"), name: "BTC-USD", assetId: .mock(.bitcoin), volume24h: 0)

        try store.upsertPerpetuals([delisted])

        try db.dbQueue.read { db in
            let browsed = try PerpetualsRequest(searchQuery: "", limit: 100, requiresVolume: true).fetch(db)
            let searched = try PerpetualsRequest(searchQuery: "BTC", limit: 100, requiresVolume: false).fetch(db)

            #expect(browsed.isEmpty)
            #expect(searched.map(\.perpetual.id) == [delisted.id])
        }
    }
}
