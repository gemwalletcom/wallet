// Copyright (c). Gem Wallet. All rights reserved.

import Store
import GemstoneServices
import Foundation
import Primitives

public protocol TransferMetadataProvidable: Sendable {
    func metadata(
        walletId: WalletId,
        assetId: AssetId,
        feeAssetId: AssetId,
        extraIds: [AssetId],
    ) throws -> TransferDataMetadata
}

public extension TransferMetadataProvidable {
    func metadata(
        wallet: Wallet,
        data: TransferData,
    ) throws -> TransferDataMetadata {
        try metadata(
            walletId: wallet.id,
            assetId: data.type.asset.id,
            feeAssetId: data.type.feeAsset.id,
            extraIds: data.type.assetIds,
        )
    }
}

public final class TransferMetadataProvider: TransferMetadataProvidable {
    private let balanceStore: BalanceStore
    private let priceStore: PriceStore

    public init(
        balanceStore: BalanceStore,
        priceStore: PriceStore,
    ) {
        self.balanceStore = balanceStore
        self.priceStore = priceStore
    }

    public func metadata(
        walletId: WalletId,
        assetId: AssetId,
        feeAssetId: AssetId,
        extraIds: [AssetId] = [],
    ) throws -> TransferDataMetadata {
        guard let balance = try balanceStore.getBalance(walletId: walletId, assetId: assetId) else {
            throw AnyError("Missing balance for assetId: \(assetId.identifier)")
        }
        guard let feeBalance = try balanceStore.getBalance(walletId: walletId, assetId: feeAssetId) else {
            throw AnyError("Missing balance for feeAssetId: \(feeAssetId.identifier)")
        }

        let ids = Array(Set([assetId, feeAssetId] + extraIds))
        let pricesList = try priceStore.getPrices(for: ids.map(\.identifier))
        let prices = Dictionary(uniqueKeysWithValues: pricesList.map { ($0.assetId, $0.mapToPrice()) })

        return TransferDataMetadata(
            assetId: assetId,
            feeAssetId: feeAssetId,
            assetBalance: balance,
            assetFeeBalance: feeBalance,
            assetPrices: prices,
        )
    }
}
