// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives
import Store

public struct ImportAssetsService: Sendable {
    let assetStore: AssetStore
    let preferences: Preferences

    public init(
        assetStore: AssetStore,
        preferences: Preferences,
    ) {
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

}
