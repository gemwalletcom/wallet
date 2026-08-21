// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public protocol FeeAssetProvidable: Sendable {
    func feeAssets(walletId: WalletId, chain: Chain) async throws -> [AssetData]
    func getAssetData(walletId: WalletId, assetId: AssetId) throws -> AssetData
}
