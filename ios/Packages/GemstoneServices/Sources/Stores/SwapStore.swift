// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import enum Gemstone.GemAssetFilter
import struct Gemstone.GemSwapPair
import protocol Gemstone.GemSwapStore
import enum Gemstone.RecentActivityType
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneSwapStore: GemSwapStore, @unchecked Sendable {
    private let assetStore: AssetStore
    private let transactionStore: TransactionStore
    private let recentActivityStore: RecentActivityStore

    public init(
        assetStore: AssetStore,
        transactionStore: TransactionStore,
        recentActivityStore: RecentActivityStore,
    ) {
        self.assetStore = assetStore
        self.transactionStore = transactionStore
        self.recentActivityStore = recentActivityStore
    }

    public func getSwapPairs(walletId: WalletId) async throws -> [Gemstone.GemSwapPair] {
        try transactionStore.getSwapHistory(walletId: walletId)
            .map { GemSwapPair(fromAssetId: $0.fromAsset, toAssetId: $0.toAsset) }
    }

    public func getRecentAssetIds(walletId: WalletId, types: [Gemstone.RecentActivityType], filters: [GemAssetFilter], limit: UInt32) async throws -> [Gemstone.AssetId] {
        try recentActivityStore.getRecent(
            walletId: walletId,
            types: types.map { $0.toPrimitives() },
            limit: Int(limit),
            filters: filters.map { $0.map() },
        ).map(\.asset.id)
    }

    public func getAssetIds(walletId: WalletId, filters: [GemAssetFilter], limit: UInt32) async throws -> [Gemstone.AssetId] {
        try assetStore.getAssetsData(
            walletId: walletId,
            filters: filters.map { $0.map() },
            limit: Int(limit),
        ).map(\.asset.id)
    }
}
