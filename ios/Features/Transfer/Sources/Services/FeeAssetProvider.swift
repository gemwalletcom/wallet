// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BalanceService
import GemstonePrimitives
import Primitives

public protocol FeeAssetProvidable: Sendable {
    func balance(wallet: Wallet, feeAsset: Asset) async throws -> Balance
}

public struct FeeAssetProvider: FeeAssetProvidable {
    private let assetsService: AssetsService
    private let balanceService: BalanceService

    public init(
        assetsService: AssetsService,
        balanceService: BalanceService,
    ) {
        self.assetsService = assetsService
        self.balanceService = balanceService
    }

    public func balance(wallet: Wallet, feeAsset: Asset) async throws -> Balance {
        let feeAssetId = feeAsset.id
        if let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) {
            return balance
        }

        try assetsService.addAssets(assets: [feeAsset.defaultBasic])
        try assetsService.addBalanceIfMissing(walletId: wallet.id, assetId: feeAssetId)
        await balanceService.updateBalance(for: wallet, assetIds: [feeAssetId])
        guard let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) else {
            throw AnyError("Missing balance for feeAssetId: \(feeAssetId.identifier)")
        }
        return balance
    }
}
