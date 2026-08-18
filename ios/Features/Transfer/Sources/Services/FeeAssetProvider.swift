// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BalanceService
import GemstonePrimitives
import Primitives

public protocol FeeAssetProvidable: Sendable {
    func load(wallet: Wallet, feeAssetId: AssetId) async throws -> (Asset, Balance)
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

    public func load(wallet: Wallet, feeAssetId: AssetId) async throws -> (Asset, Balance) {
        let feeAsset = try await assetsService.getOrFetchTokenAsset(for: feeAssetId)
        if let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) {
            return (feeAsset, balance)
        }

        try balanceService.addAssetsBalancesIfMissing(assetIds: [feeAssetId], wallet: wallet, isEnabled: false)
        await balanceService.updateBalance(for: wallet, assetIds: [feeAssetId])
        guard let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) else {
            throw AnyError("Missing balance for feeAssetId: \(feeAssetId.identifier)")
        }
        return (feeAsset, balance)
    }
}
