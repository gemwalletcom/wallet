// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import GemstoneServicesTestKit
import GemAPITestKit
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct ImportAssetsServiceTests {
    @Test
    func migrateKeepsSyncedPropertiesOnExistingAssets() throws {
        let assetStore = AssetStore(db: .mock())
        let bitcoin = AssetBasic.mock(
            asset: .mock(id: AssetId(chain: .bitcoin)),
            properties: .mock(),
            score: .mock(rank: 100),
        )
        try assetStore.add(assets: [bitcoin])

        try ImportAssetsService.mock(assetStore: assetStore).migrate()

        let assets = try assetStore.getAssetsData(walletId: .mock(), filters: [.priceAlerts])
        let metadata = try #require(assets.first { $0.asset.id == bitcoin.asset.id }?.metadata)
        #expect(metadata.isBuyEnabled)
        #expect(metadata.isSellEnabled)
        #expect(metadata.isSwapEnabled)
        #expect(assets.contains { $0.asset.id == AssetId(chain: .ethereum) })
    }


    @Test
    func migrateSeedsDefaultTokensAboveDefaultTokenRank() throws {
        let assetStore = AssetStore(db: .mock())
        let usdc = try #require(Chain.solana.defaultAssets.first)

        try ImportAssetsService.mock(assetStore: assetStore).migrate()

        let assets = try assetStore.getAssetsData(walletId: .mock(), filters: [.priceAlerts])
        let metadata = try #require(assets.first { $0.asset.id == usdc.id }?.metadata)
        #expect(metadata.rankScore.asInt > AssetScore.defaultScore)
    }
}
