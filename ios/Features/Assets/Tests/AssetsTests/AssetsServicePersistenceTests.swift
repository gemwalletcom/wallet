// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import GemstoneServicesTestKit
import class Gemstone.GemPriceService
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
        let asset = Asset.mock()
        let db = DB.mockAssets(assets: [.mock(asset: asset)])
        let assetStore = AssetStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let priceStore = PriceStore(db: db)
        let fiatRateStore = FiatRateStore(db: db)

        let links = [AssetLink.mock(type: .website, url: "https://bitcoin.org")]
        let associations = [AssetAssociation(assetId: AssetId(chain: .ethereum), type: .official)]
        let assetFull = AssetFull.mock(
            asset: asset,
            links: links,
            associations: associations,
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

        try fiatRateStore.add([.mock(symbol: .eur, rate: 0.5)])

        let service = AssetsService.mock(
            assetStore: assetStore,
            balanceStore: balanceStore,
            priceService: .mock(db: db),
            assetsProvider: GemAssetsServiceMock(assetResult: assetFull, store: GemstoneAssetStore(assetStore: assetStore, balanceStore: balanceStore), price: GemPriceService.mock(db: db)),
        )

        try await service.updateAsset(assetId: asset.id, currency: Currency.eur.rawValue)

        let result = try #require(try await db.dbQueue.read { db in
            try PriceRequest(assetId: asset.id).fetch(db)
        })

        #expect(result.asset.id == asset.id)
        #expect(result.links == links)
        let storedAsset = try await db.dbQueue.read { db in
            try AssetRequest(walletId: .mock(), assetId: asset.id).fetch(db)
        }
        #expect(storedAsset.associations == associations)
        #expect(result.price?.price == 50.0)
        #expect(result.price?.priceChangePercentage24h == -5.0)
        #expect(result.market?.marketCap == 500.0)
        #expect(result.market?.marketCapFdv == 750.0)
        #expect(result.market?.totalVolume == 100.0)
        #expect(result.market?.allTimeHighValue?.value == 50.0)
        #expect(result.market?.allTimeLowValue?.value == 0.5)
    }

    @Test(arguments: [Price?.none, Price.mock(price: 0)])
    func updateAssetMapsStoredZeroToNoPriceWhenAssetResponseHasNoUsablePrice(price: Price?) async throws {
        let db = DB.mock()
        let assetStore = AssetStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let priceStore = PriceStore(db: db)
        let fiatRateStore = FiatRateStore(db: db)
        let asset = Asset.mock()

        try assetStore.add(assets: [.mock(asset: asset)])
        try fiatRateStore.add([.mock(symbol: .usd, rate: 1)])
        try priceStore.updatePrices([.mock(assetId: asset.id, price: 100, priceChangePercentage24h: 10)])

        let service = AssetsService.mock(
            assetStore: assetStore,
            balanceStore: balanceStore,
            priceService: .mock(db: db),
            assetsProvider: GemAssetsServiceMock(assetResult: .mock(asset: asset, price: price), store: GemstoneAssetStore(assetStore: assetStore, balanceStore: balanceStore), price: GemPriceService.mock(db: db)),
        )

        try await service.updateAsset(assetId: asset.id, currency: Currency.usd.rawValue)

        let result = try await db.dbQueue.read { db in
            try PriceRequest(assetId: asset.id).fetch(db)
        }

        #expect(result?.price == nil)
    }
}
