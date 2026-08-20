// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Store

public protocol FeeAssetProvidable: Sendable {
    func getAssetData(walletId: WalletId, assetId: AssetId) throws -> AssetData
}

extension AssetStore: FeeAssetProvidable {}
