// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PerpetualQueryTests {
    private let wallet = Wallet.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .smartChain), .mock(chain: .tron), .mock(chain: .ethereum)])
    private let walletAssets: [AssetBasic] = [
        .mock(asset: .mock(name: "Bitcoin", symbol: "BTC", decimals: 8), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(asset: .mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6), properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true)),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
        .mock(
            asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20),
            properties: .mock(isEnabled: true, isBuyable: true, isSellable: true, isSwapable: true, isStakeable: true, stakingApr: 13.5, hasImage: true),
        ),
    ]
    private let walletBalances: [UpdateBalance] = [
        .mock(assetId: .mock(chain: .smartChain), available: 1),
        .mock(assetId: .mock(chain: .tron), available: 2),
        .mock(assetId: .mock(chain: .ethereum), available: 3),
        .mock(assetId: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), available: 4),
    ]

    @Test
    func load() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = PerpetualStore(db: db)
        let eth = AssetId(chain: .ethereum)
        let perpetual = Perpetual.mock(assetId: eth, price: 2500.0, maxLeverage: 100, isIsolatedOnly: true)

        try store.upsertPerpetuals([perpetual])
        try store.setPinned(for: [perpetual.id.identifier], value: true)

        try db.dbQueue.read { db in
            let result = try PerpetualQuery(assetId: eth).fetch(db)
            let notFound = try PerpetualQuery(assetId: AssetId(chain: .bitcoin)).fetch(db)

            #expect(result.perpetual.id == perpetual.id)
            #expect(result.perpetual.price == 2500.0)
            #expect(result.perpetual.isIsolatedOnly == true)
            #expect(result.asset.id == eth)
            #expect(result.metadata.isPinned == true)
            #expect(notFound == .empty)
        }
    }

    @Test
    func fetchAfterUpdatingExistingPerpetualIdentityFields() throws {
        let oldAssetId = AssetId(chain: .ethereum)
        let newAssetId = AssetId(chain: .hyperCore, tokenId: "perpetual::ETH")
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: oldAssetId.chain), .mock(chain: newAssetId.chain)])], assets: [
            .mock(asset: .mock(id: oldAssetId, name: "Ethereum", symbol: "ETH", decimals: 18, type: .native)),
            .mock(asset: .mock(id: newAssetId, name: "ETH", symbol: "ETH", decimals: 8, type: .perpetual)),
        ])
        let store = PerpetualStore(db: db)

        let existing = Perpetual.mock(
            id: PerpetualId(provider: .hypercore, symbol: "ETH"),
            name: "ETH",
            assetId: oldAssetId,
            isIsolatedOnly: false,
        )
        let updated = Perpetual.mock(
            id: existing.id,
            name: "ETH-PERP",
            assetId: newAssetId,
            price: 3000.0,
            maxLeverage: 75,
            isIsolatedOnly: true,
        )

        try store.upsertPerpetuals([existing])
        try store.setPinned(for: [existing.id.identifier], value: true)
        try store.upsertPerpetuals([updated])

        try db.dbQueue.read { db in
            let result = try PerpetualQuery(assetId: newAssetId).fetch(db)
            let stale = try PerpetualQuery(assetId: oldAssetId).fetch(db)

            #expect(result.perpetual.id == updated.id)
            #expect(result.perpetual.assetId == newAssetId)
            #expect(result.perpetual.name == "ETH-PERP")
            #expect(result.perpetual.price == 3000.0)
            #expect(result.perpetual.maxLeverage == 75)
            #expect(result.perpetual.isIsolatedOnly == true)
            #expect(result.asset.id == newAssetId)
            #expect(result.metadata.isPinned == true)
            #expect(stale == .empty)
        }
    }

    @Test
    func updatesMarketData() throws {
        let db = DB.mock(wallets: [wallet], assets: walletAssets, balances: walletBalances)
        let store = PerpetualStore(db: db)
        let eth = AssetId(chain: .ethereum)
        let perpetual = Perpetual.mock(
            id: PerpetualId(provider: .hypercore, symbol: "ETH"),
            name: "ETH",
            assetId: eth,
        )

        try store.upsertPerpetuals([perpetual])
        try store.updateMarket(
            coin: "ETH",
            price: 2236.45,
            pricePercentChange24h: 5.12,
            openInterest: 1_538_967.4595,
            volume24h: 1_169_046.29406,
            funding: 0.00125,
        )

        try db.dbQueue.read { db in
            let result = try PerpetualQuery(assetId: eth).fetch(db)

            #expect(result.perpetual.price == 2236.45)
            #expect(result.perpetual.pricePercentChange24h == 5.12)
            #expect(result.perpetual.openInterest == 1_538_967.4595)
            #expect(result.perpetual.volume24h == 1_169_046.29406)
            #expect(result.perpetual.funding == 0.00125)
        }
    }
}
