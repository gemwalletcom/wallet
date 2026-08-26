// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public struct AssetSearchService: Sendable {
    private let assetsService: AssetsService
    private let searchStore: SearchStore

    public init(
        assetsService: AssetsService,
        searchStore: SearchStore,
    ) {
        self.assetsService = assetsService
        self.searchStore = searchStore
    }

    public func searchAssets(wallet: Wallet, query: String) async throws -> [AssetBasic] {
        let assets = try await assetsService.searchAssets(
            query: query,
            chains: WalletSearchScope.chains(for: wallet),
        )

        try assetsService.addAssets(assets: assets)

        try searchStore.add(type: .asset, query: query, ids: assets.map(\.asset.id.identifier))

        try assetsService.addBalancesIfMissing(
            walletId: wallet.id,
            assetIds: assets.map(\.asset.id),
        )

        return assets
    }
}
