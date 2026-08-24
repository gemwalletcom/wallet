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
        switch chain {
        case .tempo:
            let assets = try assetStore.getAssetsData(
                walletId: walletId,
                filters: [.chains([chain.rawValue]), .hasBalance],
            )
            let supportedAssetIds = Set(chain.defaultAssets.map(\.id))
            return assets.filter { supportedAssetIds.contains($0.asset.id) && $0.balance.available > .zero }
        default:
            return []
        }
    }

    func getAssetData(walletId: WalletId, assetId: AssetId) throws -> AssetData {
        try assetStore.getAssetData(walletId: walletId, assetId: assetId)
    }
}
