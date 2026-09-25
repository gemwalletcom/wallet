// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PerpetualsQueryTests {
    @Test
    func browsingListsTradedMarketsAndKeepsAPinnedOneOutsideTheCap() throws {
        let db = DB.mock(
            wallets: [.mock(accounts: [.mock(chain: .ethereum), .mock(chain: .bitcoin)])],
            assets: [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)), .mock(asset: .mock(id: .mock(chain: .bitcoin), symbol: "BTC"))],
        )
        let store = PerpetualStore(db: db)
        let traded = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "ETH"), assetId: .mock(chain: .ethereum), volume24h: 10)
        let delisted = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "BTC"), name: "BTC-USD", assetId: .mock(chain: .bitcoin), volume24h: 0)

        try store.upsertPerpetuals([traded, delisted])

        try db.dbQueue.read { db in
            let markets = try PerpetualsQuery(searchQuery: "", limit: 1, requiresVolume: true).fetch(db)
            #expect(markets.map(\.perpetual.id) == [traded.id], "a market nobody trades is not offered for discovery")
        }

        try store.setPinned(for: [delisted.id.identifier], value: true)

        try db.dbQueue.read { db in
            let markets = try PerpetualsQuery(searchQuery: "", limit: 1, requiresVolume: true).fetch(db)
            #expect(markets.map(\.perpetual.id) == [delisted.id], "a pinned market is read before the cap cuts the list")
        }
    }

    @Test
    func aSearchReachesAMarketThatHasNotTraded() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .bitcoin)])], assets: [.mock(asset: .mock(id: .mock(chain: .bitcoin), symbol: "BTC"))])
        let store = PerpetualStore(db: db)
        let delisted = Perpetual.mock(id: PerpetualId(provider: .hypercore, symbol: "BTC"), name: "BTC-USD", assetId: .mock(chain: .bitcoin), volume24h: 0)

        try store.upsertPerpetuals([delisted])

        try db.dbQueue.read { db in
            let browsed = try PerpetualsQuery(searchQuery: "", limit: 100, requiresVolume: true).fetch(db)
            let searched = try PerpetualsQuery(searchQuery: "BTC", limit: 100, requiresVolume: false).fetch(db)

            #expect(browsed.isEmpty)
            #expect(searched.map(\.perpetual.id) == [delisted.id])
        }
    }
}
