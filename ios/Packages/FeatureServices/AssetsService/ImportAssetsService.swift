// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import GemstonePrimitives
import Preferences
import Primitives
import Store

public struct ImportAssetsService: Sendable {
    let assetListService: any GemAPIAssetsListService
    let assetsService: AssetsService
    let assetStore: AssetStore
    let preferences: Preferences

    public init(
        assetListService: any GemAPIAssetsListService,
        assetsService: AssetsService,
        assetStore: AssetStore,
        preferences: Preferences,
    ) {
        self.assetListService = assetListService
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.preferences = preferences
    }

    /// sync
    public func migrate() throws {
        let releaseVersion = Bundle.main.buildVersionNumber

        let chains = AssetConfiguration.allChains
        let tokenAssets = chains.flatMap(\.defaultAssets)
        let assetIds = chains.map(\.id) + tokenAssets.ids

        let existingAssetIds = try assetStore.getAssets(for: assetIds).ids.asSet()
        let missingAssetIds = assetIds.asSet().subtracting(existingAssetIds)
        let isNewVersion = preferences.localAssetsVersion < releaseVersion

        #if targetEnvironment(simulator)
        #else
            guard isNewVersion || missingAssetIds.isNotEmpty else { return }
        #endif

        if missingAssetIds.isNotEmpty {
            let chainAssets = chains
                .filter { missingAssetIds.contains($0.id) }
                .map { AssetBasic.native($0.asset) }
            let defaultTokenAssets = tokenAssets
                .filter { missingAssetIds.contains($0.id.identifier) }
                .map { AssetBasic.seed($0) }

            try assetStore.add(assets: chainAssets)
            try assetStore.insert(assets: defaultTokenAssets)
        }

        try assetStore.setAssetIsStakeable(for: chains.filter(\.isStakeSupported).map(\.id), value: true)

        #if targetEnvironment(simulator)
        #else
            preferences.localAssetsVersion = releaseVersion
        #endif
    }

    public func updateFiatAssets() async throws {
        async let getBuyAssets = try assetListService.getBuyableFiatAssets()
        async let getSellAssets = try assetListService.getSellableFiatAssets()

        let (buyAssets, sellAssets) = try await (getBuyAssets, getSellAssets)

        let assetIds = (buyAssets.assetIds + sellAssets.assetIds).compactMap { try? AssetId(id: $0) }

        try await assetsService.prefetchAssets(assetIds: assetIds)
        try assetStore.updateBuyableAssets(assetIds: buyAssets.assetIds)
        try assetStore.updateSellableAssets(assetIds: sellAssets.assetIds)

        preferences.fiatOnRampAssetsVersion = Int(buyAssets.version)
        preferences.fiatOffRampAssetsVersion = Int(sellAssets.version)
    }

    public func updateSwapAssets() async throws {
        let assets = try await assetListService.getSwapAssets()

        try await assetsService.prefetchAssets(assetIds: assets.assetIds.compactMap { try? AssetId(id: $0) })
        try assetStore.setAssetIsSwappable(for: assets.assetIds, value: true)

        preferences.swapAssetsVersion = Int(assets.version)
    }
}
