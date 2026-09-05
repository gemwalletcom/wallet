// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemApprovalValue
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemFeeAsset
import Primitives

public extension Primitives.Balance {
    init(_ balance: GemAssetBalance) {
        self.init(
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
            metadata: balance.metadata.map { $0.map() },
        )
    }
}

public extension GemFeeAsset {
    func map() -> (asset: Primitives.Asset, balance: Primitives.Balance, price: Primitives.Price?) {
        (
            asset: asset.map(),
            balance: Primitives.Balance(balance),
            price: price.map { $0.map().mapToPrice() },
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

    var assetPrice: Primitives.Price? { assetPrice().map { $0.map().mapToPrice() } }
    var feePrice: Primitives.Price? { feePrice().map { $0.map().mapToPrice() } }

    var balance: Primitives.Balance { Primitives.Balance(assetBalance) }

    func price(for assetId: String) -> Primitives.Price? {
        price(assetId: assetId).map { $0.map().mapToPrice() }
    }

    func price(for assetId: Primitives.AssetId) -> Primitives.Price? {
        price(for: assetId.identifier)
    }

    var assetPrices: [Primitives.AssetId: Primitives.Price] {
        Dictionary(uniqueKeysWithValues: prices.map { (Primitives.AssetId(core: $0.assetId), $0.map().mapToPrice()) })
    }
}
