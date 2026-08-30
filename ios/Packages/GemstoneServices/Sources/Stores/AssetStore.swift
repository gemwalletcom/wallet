// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetBasic
import typealias Gemstone.AssetFull
import typealias Gemstone.AssetId
import protocol Gemstone.GemAssetStore
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneAssetStore: GemAssetStore, @unchecked Sendable {
    private let assetStore: AssetStore
    private let balanceStore: BalanceStore

    public init(assetStore: AssetStore, balanceStore: BalanceStore) {
        self.assetStore = assetStore
        self.balanceStore = balanceStore
    }

    public func getAssetIds(assetIds: [Gemstone.AssetId]) async throws -> [Gemstone.AssetId] {
        try assetStore.getAssets(for: assetIds).map(\.id.identifier)
    }

    public func getAssets(assetIds: [Gemstone.AssetId]) throws -> [Gemstone.Asset] {
        try assetStore.getAssets(for: assetIds).map { $0.json() }
    }

    public func saveAssets(assets: [Gemstone.AssetBasic]) async throws {
        try assetStore.add(assets: assets.map { try Primitives.AssetBasic($0) })
    }

    public func saveAsset(asset: Gemstone.AssetFull) async throws {
        let asset = try Primitives.AssetFull(asset)
        try assetStore.add(assets: [asset.basic])
        try assetStore.updateLinks(assetId: asset.asset.id, asset.links)
        try assetStore.updateAssociations(assetId: asset.asset.id, associations: asset.associations)
    }

    public func setBuyableAssets(assetIds: [Gemstone.AssetId]) async throws {
        try assetStore.updateBuyableAssets(assetIds: assetIds)
    }

    public func setSellableAssets(assetIds: [Gemstone.AssetId]) async throws {
        try assetStore.updateSellableAssets(assetIds: assetIds)
    }

    public func setStakeableAssets(assetIds: [Gemstone.AssetId]) async throws {
        try assetStore.setAssetIsStakeable(for: assetIds, value: true)
    }

    public func setSwappableAssets(assetIds: [Gemstone.AssetId]) async throws {
        try assetStore.setAssetIsSwappable(for: assetIds, value: true)
    }

    public func addBalances(walletId: String, assetIds: [Gemstone.AssetId], enabled: Bool) async throws {
        try balanceStore.addBalance(
            assetIds: assetIds.map { try Primitives.AssetId(id: $0) },
            isEnabled: enabled,
            for: WalletId.from(id: walletId),
        )
    }

    public func addMissingBalances(walletId: String, assetIds: [Gemstone.AssetId]) async throws {
        try balanceStore.addBalance(
            assetIds: assetIds.map { try Primitives.AssetId(id: $0) },
            isEnabled: false,
            for: WalletId.from(id: walletId),
        )
    }
}
