// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct AssetSearchService: Sendable {
    private let assetsService: any GemAssetsServiceProtocol
    private let assetStore: AssetStore
    private let searchStore: SearchStore

    public init(
        assetsService: any GemAssetsServiceProtocol,
        assetStore: AssetStore,
        searchStore: SearchStore,
    ) {
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.searchStore = searchStore
    }

    public func searchAssets(wallet: Wallet, query: String) async throws -> [AssetBasic] {
        let assets = try await assetsService.searchAssetsAndTokens(
            query: query,
            chains: WalletSearchScope.chains(for: wallet),
        )
        try assetStore.add(assets: assets)
        try searchStore.add(type: .asset, query: query, ids: assets.map(\.asset.id.identifier))
        try await assetsService.addMissingBalances(
            walletId: wallet.id,
            assetIds: assets.map(\.asset.id),
        )
        return assets
    }
}
