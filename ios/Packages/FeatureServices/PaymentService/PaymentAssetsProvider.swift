// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public struct PaymentAssetsProvider: PaymentAssetsProvidable {
    private let assetStore: AssetStore

    public init(assetStore: AssetStore) {
        self.assetStore = assetStore
    }

    public func assetsData(walletId: WalletId, assetIds: [AssetId]) throws -> [AssetData] {
        try assetStore.getAssetsData(walletId: walletId, filters: [.chainsOrAssets([], assetIds.map(\.identifier))])
    }
}
