// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct AssetStoreTests {
    @Test
    func availabilityUpdatesOnlyChangedRows() throws {
        let db = DB.mockWithChains([.ethereum, .bitcoin, .solana])
        let store = AssetStore(db: db)
        let ethereum = Chain.ethereum.assetId.identifier
        let bitcoin = Chain.bitcoin.assetId.identifier

        _ = try store.updateBuyableAssets(assetIds: [])

        #expect(try store.updateBuyableAssets(assetIds: [ethereum, bitcoin]) == 2)
        #expect(try store.updateBuyableAssets(assetIds: [ethereum, bitcoin]) == 0)
        #expect(try store.updateBuyableAssets(assetIds: [ethereum]) == 1)
    }

    @Test
    func swappableFlagIsSetOnlyWhereMissing() throws {
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let store = AssetStore(db: db)
        let assetIds = [Chain.ethereum.assetId.identifier, Chain.bitcoin.assetId.identifier]

        let first = try store.setAssetIsSwappable(for: assetIds, value: true)
        let second = try store.setAssetIsSwappable(for: assetIds, value: true)

        #expect(first + second == first)
        #expect(second == 0)
    }
}
