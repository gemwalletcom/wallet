// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BalanceService
import GemstonePrimitives
import Primitives

public protocol FeeAssetProvidable: Sendable {
    func feeAsset(wallet: Wallet, asset: Asset, fee: Fee) async throws -> (asset: Asset, balance: Balance)
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

    public func feeAsset(wallet: Wallet, asset: Asset, fee: Fee) async throws -> (asset: Asset, balance: Balance) {
        let feeAssetId = fee.feeAssetId
        return try await (
            asset: feeAsset(for: feeAssetId, asset: asset),
            balance: balance(for: feeAssetId, wallet: wallet)
        )
    }
}

// MARK: - Private

extension FeeAssetProvider {
    private func feeAsset(for feeAssetId: AssetId, asset: Asset) async throws -> Asset {
        if feeAssetId == asset.id {
            asset
        } else if feeAssetId == asset.feeAsset.id {
            asset.feeAsset
        } else {
            try await assetsService.getOrFetchTokenAsset(for: feeAssetId)
        }
    }

    private func balance(for feeAssetId: AssetId, wallet: Wallet) async throws -> Balance {
        if let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) {
            return balance
        }

        try assetsService.addBalanceIfMissing(walletId: wallet.id, assetId: feeAssetId)
        await balanceService.updateBalance(for: wallet, assetIds: [feeAssetId])
        guard let balance = try balanceService.getBalance(walletId: wallet.id, assetId: feeAssetId) else {
            throw AnyError("Missing balance for feeAssetId: \(feeAssetId.identifier)")
        }
        return balance
    }
}
