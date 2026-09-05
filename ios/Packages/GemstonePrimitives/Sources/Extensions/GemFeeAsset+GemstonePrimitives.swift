// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemApprovalValue
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAssetPrice
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemFeeAsset
import Primitives

public extension Primitives.Balance {
    init(_ balance: GemAssetBalance) throws {
        try self.init(
            available: BigInt(balance.available),
            frozen: BigInt(balance.frozen),
            locked: BigInt(balance.locked),
            staked: BigInt(balance.staked),
            pending: BigInt(balance.pending),
            pendingUnconfirmed: BigInt(balance.pendingUnconfirmed),
            rewards: BigInt(balance.rewards),
            reserved: BigInt(balance.reserved),
            withdrawable: BigInt(balance.withdrawable),
            earn: BigInt(balance.earn),
            metadata: balance.metadata.map { try BalanceMetadata($0) },
        )
    }
}

public extension GemFeeAsset {
    func map() throws -> (asset: Primitives.Asset, balance: Primitives.Balance, price: Primitives.Price?) {
        try (
            asset: asset.map(),
            balance: Primitives.Balance(balance),
            price: price.map { Primitives.Price(price: $0.price, priceChangePercentage24h: $0.priceChangePercentage24h, updatedAt: Date(timeIntervalSince1970: TimeInterval($0.updatedAt))) },
        )
    }
}

public extension Primitives.Price {
    init(_ price: GemAssetPrice) {
        self.init(
            price: price.price,
            priceChangePercentage24h: price.priceChangePercentage24h,
            updatedAt: Date(timeIntervalSince1970: TimeInterval(price.updatedAt)),
        )
    }
}

public extension GemApprovalValue {
    func map() -> Primitives.ApprovalValue {
        switch self {
        case let .exact(value): .exact(BigInt(value))
        case .unlimited: .unlimited
        }
    }
}

public extension GemConfirmMetadata {
    var assetId: Primitives.AssetId { Primitives.AssetId(core: assetBalance.assetId) }
    var feeAssetId: Primitives.AssetId { Primitives.AssetId(core: feeAssetBalance.assetId) }

    var available: BigInt { BigInt(assetBalance.available) }

    var assetPrice: Primitives.Price? { assetPrice().map { Primitives.Price($0) } }
    var feePrice: Primitives.Price? { feePrice().map { Primitives.Price($0) } }

    var balance: Primitives.Balance? { try? Primitives.Balance(assetBalance) }

    func price(for assetId: String) -> Primitives.Price? {
        price(assetId: assetId).map { Primitives.Price($0) }
    }

    func price(for assetId: Primitives.AssetId) -> Primitives.Price? {
        price(for: assetId.identifier)
    }

    var assetPrices: [Primitives.AssetId: Primitives.Price] {
        Dictionary(uniqueKeysWithValues: prices.map { (Primitives.AssetId(core: $0.assetId), Primitives.Price($0)) })
    }
}
