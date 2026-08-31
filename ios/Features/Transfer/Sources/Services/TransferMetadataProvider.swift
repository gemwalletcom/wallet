// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemTransferService
import protocol Gemstone.GemConfirmServiceProtocol
import struct Gemstone.GemAssetBalance
import Foundation
import GemstonePrimitives
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
        transferService: GemTransferService,
    ) throws -> TransferDataMetadata {
        try metadata(
            walletId: wallet.id,
            assetId: data.type.asset.id,
            feeAssetId: data.type.feeAsset(transferService: transferService).id,
            extraIds: data.type.assetIds(transferService: transferService),
        )
    }
}

public final class TransferMetadataProvider: TransferMetadataProvidable {
    private let confirmService: any GemConfirmServiceProtocol

    public init(confirmService: any GemConfirmServiceProtocol) {
        self.confirmService = confirmService
    }

    public func metadata(
        walletId: WalletId,
        assetId: AssetId,
        feeAssetId: AssetId,
        extraIds: [AssetId] = [],
    ) throws -> TransferDataMetadata {
        let metadata = try confirmService.metadata(
            walletId: walletId.id,
            assetId: assetId.identifier,
            feeAssetId: feeAssetId.identifier,
            extraAssetIds: extraIds.map(\.identifier),
        )
        let prices = try metadata.prices.map { price in
            try (AssetId(id: price.assetId), Price(price: price.price, priceChangePercentage24h: price.priceChangePercentage24h, updatedAt: Date(timeIntervalSince1970: TimeInterval(price.updatedAt))))
        }
        return TransferDataMetadata(
            assetId: assetId,
            feeAssetId: feeAssetId,
            assetBalance: try Balance(metadata.assetBalance),
            assetFeeBalance: try Balance(metadata.feeAssetBalance),
            assetPrices: Dictionary(uniqueKeysWithValues: prices),
        )
    }
}

private extension Balance {
    init(_ balance: GemAssetBalance) throws {
        self.init(
            available: try BigInt.from(string: balance.available),
            frozen: try BigInt.from(string: balance.frozen),
            locked: try BigInt.from(string: balance.locked),
            staked: try BigInt.from(string: balance.staked),
            pending: try BigInt.from(string: balance.pending),
            pendingUnconfirmed: try BigInt.from(string: balance.pendingUnconfirmed),
            rewards: try BigInt.from(string: balance.rewards),
            reserved: try BigInt.from(string: balance.reserved),
            withdrawable: try BigInt.from(string: balance.withdrawable),
            earn: try BigInt.from(string: balance.earn),
            metadata: try balance.metadata.map { try BalanceMetadata($0) },
        )
    }
}
