// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
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
    func updateFiatAssetsDisablesAssetsDroppedFromLists() async throws {
        let assetStore = AssetStore(db: .mock())
        let bitcoin = AssetId(chain: .bitcoin)
        let ethereum = AssetId(chain: .ethereum)
        try assetStore.add(assets: [
            .mock(asset: .mock(id: bitcoin), properties: .mock(), score: .mock(rank: 100)),
            .mock(asset: .mock(id: ethereum), properties: .mock(), score: .mock(rank: 100)),
        ])
        try assetStore.setAssetIsBuyable(for: [ethereum.identifier], value: false)
        try assetStore.setAssetIsSellable(for: [ethereum.identifier], value: false)
        let service = ImportAssetsService.mock(
            assetsProvider: GemAssetsServiceMock(
                buyableFiatAssets: FiatAssets(version: 7, assetIds: [ethereum.identifier]),
                sellableFiatAssets: FiatAssets(version: 9, assetIds: [ethereum.identifier]),
            ),
            assetsService: .mock(assetStore: assetStore),
            assetStore: assetStore,
        )

        try await service.updateFiatAssets()

        let assets = try assetStore.getAssetsData(walletId: .mock(), filters: [.priceAlerts])
        let bitcoinMetadata = try #require(assets.first { $0.asset.id == bitcoin }?.metadata)
        let ethereumMetadata = try #require(assets.first { $0.asset.id == ethereum }?.metadata)
        #expect(bitcoinMetadata.isBuyEnabled == false)
        #expect(bitcoinMetadata.isSellEnabled == false)
        #expect(ethereumMetadata.isBuyEnabled)
        #expect(ethereumMetadata.isSellEnabled)
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
