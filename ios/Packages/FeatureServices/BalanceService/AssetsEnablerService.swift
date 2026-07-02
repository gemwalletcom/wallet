// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import PriceService
import Primitives

public struct AssetsEnablerService: AssetsEnabler {
    private let assetsService: AssetsService
    private let balanceUpdater: any BalanceUpdater
    private let priceUpdater: any PriceUpdater

    public init(
        assetsService: AssetsService,
        balanceUpdater: any BalanceUpdater,
        priceUpdater: any PriceUpdater,
    ) {
        self.assetsService = assetsService
        self.balanceUpdater = balanceUpdater
        self.priceUpdater = priceUpdater
    }

    public func enableAssets(wallet: Wallet, assetIds: [AssetId], enabled: Bool) async throws {
        let walletId = wallet.id
        let requestedAssetIds = assetIds.unique()
        guard !requestedAssetIds.isEmpty else { return }

        let enabledAssetIds = try assetsService
            .getBalanceAssetIds(walletId: walletId, assetIds: requestedAssetIds, filters: [.enabled])
            .asSet()

        for assetId in requestedAssetIds {
            try assetsService.addBalanceIfMissing(walletId: walletId, assetId: assetId)
        }

        try assetsService.updateEnabled(walletId: walletId, assetIds: requestedAssetIds, enabled: enabled)

        guard enabled else { return }

        let assetIds = requestedAssetIds.filter { !enabledAssetIds.contains($0) }
        guard !assetIds.isEmpty else { return }

        async let balanceUpdate: () = balanceUpdater.updateBalance(for: wallet, assetIds: assetIds)
        async let priceUpdate: () = priceUpdater.addPrices(assetIds: assetIds)
        _ = await balanceUpdate
        _ = try await priceUpdate
    }

    public func enableAssetId(wallet: Wallet, assetId: AssetId) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try await enableAssets(wallet: wallet, assetIds: [asset.id], enabled: true)
    }
}
