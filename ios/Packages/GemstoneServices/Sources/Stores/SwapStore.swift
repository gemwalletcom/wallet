// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import struct Gemstone.GemSwapPair
import protocol Gemstone.GemSwapStore
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

    public func getSwapPairs(walletId: String) async throws -> [Gemstone.GemSwapPair] {
        try transactionStore.getSwapHistory(walletId: WalletId.from(id: walletId))
            .map { GemSwapPair(fromAssetId: $0.fromAsset.identifier, toAssetId: $0.toAsset.identifier) }
    }

    public func getRecentAssetIds(walletId: String) async throws -> [Gemstone.AssetId] {
        try recentActivityStore.getRecent(walletId: WalletId.from(id: walletId), types: [.swapSelect, .swap])
            .map(\.asset.id.identifier)
    }

    public func getPayAssetIds(walletId: String) async throws -> [Gemstone.AssetId] {
        try assetStore.getAssetsData(
            walletId: WalletId.from(id: walletId),
            filters: [.enabled, .swappable],
            limit: nil,
        ).map(\.asset.id.identifier)
    }

    public func getReceiveAssetIds(walletId: String, chains: [Gemstone.Chain], assetIds: [Gemstone.AssetId]) async throws -> [Gemstone.AssetId] {
        try assetStore.getAssetsData(
            walletId: WalletId.from(id: walletId),
            filters: [.enabled, .swappable, .chainsOrAssets(chains, assetIds)],
            limit: nil,
        ).map(\.asset.id.identifier)
    }
}
