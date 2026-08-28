// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import Store

struct FeeAssetProvider: FeeAssetProvidable {
    private let assetStore: AssetStore

    init(assetStore: AssetStore) {
        self.assetStore = assetStore
    }

    func feeAssets(walletId: WalletId, chain: Chain) async throws -> [AssetData] {
        let feeAssetIds = Set(chain.feeAssetIds)
        guard feeAssetIds.isNotEmpty else { return [] }
        let assets = try assetStore.getAssetsData(
            walletId: walletId,
            filters: [.chains([chain.rawValue]), .hasBalance],
        )
        return assets.filter { feeAssetIds.contains($0.asset.id) && $0.balance.available > .zero }
    }

    func getAssetData(walletId: WalletId, assetId: AssetId) throws -> AssetData {
        try assetStore.getAssetData(walletId: walletId, assetId: assetId)
    }
}
