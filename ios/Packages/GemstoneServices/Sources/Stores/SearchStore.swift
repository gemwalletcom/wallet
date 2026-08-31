// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.AssetList
import protocol Gemstone.GemSearchStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneSearchStore: GemSearchStore, @unchecked Sendable {
    private let store: SearchStore
    private let assetListStore: AssetListStore

    public init(store: SearchStore, assetListStore: AssetListStore) {
        self.store = store
        self.assetListStore = assetListStore
    }

    public func setAssets(key: String, assetIds: [Gemstone.AssetId]) async throws {
        try store.add(type: .asset, query: key, ids: assetIds)
    }

    public func setPerpetuals(key: String, perpetualIds: [String]) async throws {
        try store.add(type: .perpetual, query: key, ids: perpetualIds)
    }

    public func setLists(key: String, lists: [Gemstone.AssetList]) async throws {
        let lists = try lists.map { try Primitives.AssetList($0) }
        try assetListStore.upsert(lists)
        try store.add(type: .list, query: key, ids: lists.map(\.id))
    }
}
