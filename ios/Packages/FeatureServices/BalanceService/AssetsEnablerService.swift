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

        if enabled {
            try await assetsService.prefetchAssets(assetIds: requestedAssetIds)
        }

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

    public func pinAsset(wallet: Wallet, assetId: AssetId, pinned: Bool) async throws {
        if pinned {
            try await enableAssets(wallet: wallet, assetIds: [assetId], enabled: true)
        }

        try assetsService.updatePinned(walletId: wallet.id, assetId: assetId, pinned: pinned)
    }
}
