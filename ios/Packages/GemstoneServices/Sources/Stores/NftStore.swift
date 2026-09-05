// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.NftAssetData
import typealias Gemstone.NftAssetId
import struct Gemstone.NftData
import protocol Gemstone.GemNftStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneNftStore: GemNftStore, @unchecked Sendable {
    private let store: NftStore

    public init(store: NftStore) {
        self.store = store
    }

    public func saveNfts(walletId: String, data: [Gemstone.NftData]) async throws {
        try store.save(data.map { $0.map() }, for: WalletId.from(id: walletId))
    }

    public func getAssetData(assetId: Gemstone.NftAssetId) async throws -> Gemstone.NftAssetData? {
        guard let asset = try store.getAsset(assetId: NFTAssetId.from(id: assetId)),
              let collection = try store.getCollection(collectionId: asset.collectionId)
        else {
            return nil
        }
        return NFTAssetData(collection: collection, asset: asset).map()
    }

    public func saveAsset(data: Gemstone.NftAssetData) async throws {
        let data = data.map()
        try store.add(asset: data.asset, collection: data.collection)
    }
}
