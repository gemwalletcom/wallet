// Copyright (c). Gem Wallet. All rights reserved.

import AssetsServiceTestKit
import BalanceService
import BalanceServiceTestKit
import Foundation
import GemAPI
import GemAPITestKit
import PriceServiceTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetsEnablerServiceTests {
    private let assetId = AssetId(chain: .bitcoin)
    private let unknownAssetId = AssetId(chain: .ethereum, tokenId: "0xunknown")

    @Test
    func pinningHiddenAssetShowsIt() async throws {
        let db = DB.mockAssets()
        let service = makeService(db: db)
        try BalanceStore.mock(db: db).setIsEnabled(walletId: .mock(), assetIds: [assetId], value: false)

        try await service.pinAsset(wallet: .mock(), assetId: assetId, pinned: true)

        let metadata = try metadata(in: db, assetId: assetId)
        #expect(metadata?.isPinned == true)
        #expect(metadata?.isBalanceEnabled == true)
    }

    @Test
    func unpinningHiddenAssetKeepsItHidden() async throws {
        let db = DB.mockAssets()
        let balanceStore = BalanceStore.mock(db: db)
        let service = makeService(db: db)
        try balanceStore.pinAsset(walletId: .mock(), assetId: assetId, value: true)
        try balanceStore.setIsEnabled(walletId: .mock(), assetIds: [assetId], value: false)

        try await service.pinAsset(wallet: .mock(), assetId: assetId, pinned: false)

        let metadata = try metadata(in: db, assetId: assetId)
        #expect(metadata?.isPinned == false)
        #expect(metadata?.isBalanceEnabled == false)
    }

    @Test
    func hidingAssetUnpinsItAndKeepsBalanceRow() async throws {
        let db = DB.mockAssets()
        let balanceStore = BalanceStore.mock(db: db)
        let service = makeService(db: db)
        try balanceStore.pinAsset(walletId: .mock(), assetId: assetId, value: true)

        try await service.enableAssets(wallet: .mock(), assetIds: [assetId], enabled: false)

        let metadata = try metadata(in: db, assetId: assetId)
        #expect(metadata?.isBalanceEnabled == false)
        #expect(metadata?.isPinned == false)
        #expect(try balanceStore.isBalanceExist(walletId: .mock(), assetId: assetId))
    }

    @Test
    func enablingUnknownAssetFetchesIt() async throws {
        let db = DB.mockAssets()
        let service = makeService(db: db, assetsProvider: providerReturningUnknownAsset)

        try await service.enableAssets(wallet: .mock(), assetIds: [unknownAssetId], enabled: true)

        let storedIds = try AssetStore.mock(db: db).getAssets(for: [unknownAssetId.identifier]).map(\.id)
        #expect(storedIds == [unknownAssetId])
        #expect(try metadata(in: db, assetId: unknownAssetId)?.isBalanceEnabled == true)
    }

    @Test
    func pinningUnknownAssetShowsAndPinsIt() async throws {
        let db = DB.mockAssets()
        let service = makeService(db: db, assetsProvider: providerReturningUnknownAsset)

        try await service.pinAsset(wallet: .mock(), assetId: unknownAssetId, pinned: true)

        let metadata = try metadata(in: db, assetId: unknownAssetId)
        #expect(metadata?.isPinned == true)
        #expect(metadata?.isBalanceEnabled == true)
    }

    private var providerReturningUnknownAsset: GemAPIAssetsServiceMock {
        GemAPIAssetsServiceMock(assetsResult: [.mock(asset: .mock(id: unknownAssetId))])
    }

    private func makeService(
        db: DB,
        assetsProvider: any GemAPIAssetsService = GemAPIAssetsServiceMock(),
    ) -> AssetsEnablerService {
        AssetsEnablerService(
            assetsService: .mock(
                assetStore: .mock(db: db),
                balanceStore: .mock(db: db),
                assetsProvider: assetsProvider,
            ),
            balanceUpdater: BalanceUpdaterMock(),
            priceUpdater: PriceUpdaterMock(),
        )
    }

    private func metadata(in db: DB, assetId: AssetId) throws -> AssetMetaData? {
        try db.dbQueue.read { db in
            try AssetsRequest(walletId: .mock(), searchBy: "", filters: [])
                .fetch(db)
                .first { $0.asset.id == assetId }?
                .metadata
        }
    }
}
