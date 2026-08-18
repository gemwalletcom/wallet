// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Primitives

public protocol FeeAssetProvidable: Sendable {
    func load(walletId: WalletId, feeAssetId: AssetId) throws -> AssetData
}

public struct FeeAssetProvider: FeeAssetProvidable {
    private let assetsService: AssetsService

    public init(assetsService: AssetsService) {
        self.assetsService = assetsService
    }

    public func load(walletId: WalletId, feeAssetId: AssetId) throws -> AssetData {
        try assetsService.assetStore.getAssetData(walletId: walletId, assetId: feeAssetId)
    }
}
