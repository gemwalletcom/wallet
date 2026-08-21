// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import Transfer

struct FeeAssetProviderTests {
    @Test
    func returnsFundedDefaultTempoAssets() async throws {
        let funded = Array(Chain.tempo.defaultAssets.dropLast())
        let unfunded = try #require(Chain.tempo.defaultAssets.last)
        let unsupported = Asset.mock(id: AssetId(chain: .tempo, tokenId: "0x1"), type: .tip20)
        let tempoNative = Asset.mock(id: AssetId(chain: .tempo))
        let storedAssets = ([unfunded] + funded + [unsupported, tempoNative]).map { AssetBasic.mock(asset: $0) }
        let db = DB.mockAssets(assets: storedAssets)

        let assets = try await FeeAssetProvider(assetStore: AssetStore(db: db))
            .feeAssets(walletId: .mock(), chain: .tempo)

        #expect(Set(assets.map(\.asset.id)) == Set(funded.map(\.id)))
    }

    @Test
    func returnsNoAssetsForUnsupportedChain() async throws {
        let assets = try await FeeAssetProvider(assetStore: AssetStore(db: .mockAssets()))
            .feeAssets(walletId: .mock(), chain: .ethereum)

        #expect(assets.isEmpty)
    }
}
