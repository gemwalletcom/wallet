// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.NftAssetData
import typealias Gemstone.NftAssetId
import typealias Gemstone.NftData
import protocol Gemstone.GemNftStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneNftStore: GemNftStore, @unchecked Sendable {
    private let store: NFTStore

    public init(store: NFTStore) {
        self.store = store
    }

    public func save(walletId: String, data: [Gemstone.NftData]) async throws {
        try store.save(data.map { try NFTData($0) }, for: WalletId.from(id: walletId))
    }

    public func getAssetData(assetId: Gemstone.NftAssetId) async throws -> Gemstone.NftAssetData? {
        guard let asset = try store.getAsset(assetId: NFTAssetId.from(id: assetId)),
              let collection = try store.getCollection(collectionId: asset.collectionId)
        else {
            return nil
        }
        return try NFTAssetData(collection: collection, asset: asset).json()
    }

    public func saveAsset(data: Gemstone.NftAssetData) async throws {
        let data = try NFTAssetData(data)
        try store.add(asset: data.asset, collection: data.collection)
    }
}
