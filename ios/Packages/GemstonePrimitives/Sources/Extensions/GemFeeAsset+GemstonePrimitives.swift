// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAssetPrice
import struct Gemstone.GemConfirmMetadata
import enum Gemstone.GemApprovalValue
import struct Gemstone.GemFeeAsset
import Primitives

public extension Primitives.Balance {
    init(_ balance: GemAssetBalance) throws {
        try self.init(
            available: BigInt.from(string: balance.available),
            frozen: BigInt.from(string: balance.frozen),
            locked: BigInt.from(string: balance.locked),
            staked: BigInt.from(string: balance.staked),
            pending: BigInt.from(string: balance.pending),
            pendingUnconfirmed: BigInt.from(string: balance.pendingUnconfirmed),
            rewards: BigInt.from(string: balance.rewards),
            reserved: BigInt.from(string: balance.reserved),
            withdrawable: BigInt.from(string: balance.withdrawable),
            earn: BigInt.from(string: balance.earn),
            metadata: balance.metadata.map { try BalanceMetadata($0) },
        )
    }
}

public extension GemFeeAsset {
    func map() throws -> (asset: Primitives.Asset, balance: Primitives.Balance, price: Primitives.Price?) {
        try (
            asset: asset.map(),
            balance: Primitives.Balance(balance),
            price: price.map { Primitives.Price(price: $0.price, priceChangePercentage24h: $0.priceChangePercentage24h, updatedAt: Date(timeIntervalSince1970: TimeInterval($0.updatedAt))) }
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
    func map() throws -> Primitives.ApprovalValue {
        switch self {
        case let .exact(value): .exact(try BigInt.from(string: value))
        case .unlimited: .unlimited
        }
    }
}

public extension GemConfirmMetadata {
    var assetId: Primitives.AssetId? { try? Primitives.AssetId(id: assetBalance.assetId) }
    var feeAssetId: Primitives.AssetId? { try? Primitives.AssetId(id: feeAssetBalance.assetId) }

    var available: BigInt { (try? BigInt.from(string: assetBalance.available)) ?? .zero }
    var feeAvailable: String { feeAssetBalance.available }

    var assetPrice: Primitives.Price? { price(for: assetBalance.assetId) }
    var feePrice: Primitives.Price? { price(for: feeAssetBalance.assetId) }

    var balance: Primitives.Balance? { try? Primitives.Balance(assetBalance) }
    var feeBalance: Primitives.Balance? { try? Primitives.Balance(feeAssetBalance) }

    func price(for assetId: String) -> Primitives.Price? {
        prices.first { $0.assetId == assetId }.map { Primitives.Price($0) }
    }

    func price(for assetId: Primitives.AssetId) -> Primitives.Price? {
        price(for: assetId.identifier)
    }
}

public extension GemConfirmMetadata {
    var assetPrices: [Primitives.AssetId: Primitives.Price] {
        Dictionary(uniqueKeysWithValues: prices.compactMap { price in
            (try? Primitives.AssetId(id: price.assetId)).map { ($0, Primitives.Price(price)) }
        })
    }
}
