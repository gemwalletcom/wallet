// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import Foundation
import GemAPITestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetsServicePersistenceTests {
    @Test
    func updateAssetStoresPriceMarketAndLinksFromAssetResponse() async throws {
        let db = DB.mock()
        let assetStore = AssetStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let priceStore = PriceStore(db: db)
        let fiatRateStore = FiatRateStore(db: db)

        let asset = Asset.mock()
        let links = [AssetLink.mock(type: .website, url: "https://bitcoin.org")]
        let assetFull = AssetFull.mock(
            asset: asset,
            links: links,
            price: .mock(price: 100.0, priceChangePercentage24h: -5.0),
            market: .mock(
                marketCap: 1000.0,
                marketCapFdv: 1500.0,
                marketCapRank: 1,
                totalVolume: 200.0,
                circulatingSupply: 10.0,
                totalSupply: 20.0,
                maxSupply: 21.0,
                allTimeHighValue: .init(date: .now, value: 100.0, percentage: -10.0),
                allTimeLowValue: .init(date: .now, value: 1.0, percentage: 25.0),
            ),
        )

        try fiatRateStore.add([.mock(symbol: Currency.eur.rawValue, rate: 0.5)])

        let service = AssetsService.mock(
            assetStore: assetStore,
            balanceStore: balanceStore,
            priceStore: priceStore,
            assetsProvider: GemAPIAssetsServiceMock(assetResult: assetFull),
        )

        try await service.updateAsset(assetId: asset.id, currency: Currency.eur.rawValue)

        let result = try #require(try await db.dbQueue.read { db in
            try PriceRequest(assetId: asset.id).fetch(db)
        })

        #expect(result.asset.id == asset.id)
        #expect(result.links == links)
        #expect(result.price?.price == 50.0)
        #expect(result.price?.priceChangePercentage24h == -5.0)
        #expect(result.market?.marketCap == 500.0)
        #expect(result.market?.marketCapFdv == 750.0)
        #expect(result.market?.totalVolume == 100.0)
        #expect(result.market?.allTimeHighValue?.value == 50.0)
        #expect(result.market?.allTimeLowValue?.value == 0.5)
    }
}
