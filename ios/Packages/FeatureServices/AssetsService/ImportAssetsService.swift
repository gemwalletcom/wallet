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
        let defaultAssets = chains.map(\.defaultAssets).flatMap(\.self)
        let assetIds = chains.map(\.id) + defaultAssets.ids

        let existingAssets = try assetStore.getAssets(for: assetIds)
        let hasMissingAssets = existingAssets.count != assetIds.count
        let isNewVersion = preferences.localAssetsVersion < releaseVersion

        #if targetEnvironment(simulator)
        #else
            guard isNewVersion || hasMissingAssets else { return }
        #endif

        if hasMissingAssets {
            let chainAssets = chains.map { AssetBasic.native($0.asset) }
            let defaultTokenAssets = defaultAssets.map { AssetBasic.seed($0) }

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

        async let prefetchResult = try assetsService.prefetchAssets(assetIds: assetIds)
        async let setBuyableResult = try assetStore.setAssetIsBuyable(for: buyAssets.assetIds, value: true)
        async let setSellableResult = try assetStore.setAssetIsSellable(for: sellAssets.assetIds, value: true)

        _ = try await (prefetchResult, setBuyableResult, setSellableResult)

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
