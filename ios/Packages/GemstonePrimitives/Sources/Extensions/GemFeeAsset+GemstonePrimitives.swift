// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
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
            metadata: balance.metadata.map { $0.toPrimitives() },
        )
    }
}

public extension GemFeeAsset {
    func toPrimitives() -> (asset: Primitives.Asset, balance: Primitives.Balance, price: Primitives.Price?) {
        (
            asset: asset.toPrimitives(),
            balance: Primitives.Balance(balance),
            price: price.map { $0.toPrimitives().mapToPrice() },
        )
    }
}

public extension GemConfirmMetadata {
    var assetId: Primitives.AssetId { Primitives.AssetId(core: assetBalance.assetId) }
    var feeAssetId: Primitives.AssetId { Primitives.AssetId(core: feeAssetBalance.assetId) }

    var available: BigInt { BigInt(assetBalance.available) }

    var assetPrice: Primitives.Price? { assetPrice().map { $0.toPrimitives().mapToPrice() } }
    var feePrice: Primitives.Price? { feePrice().map { $0.toPrimitives().mapToPrice() } }

    var balance: Primitives.Balance { Primitives.Balance(assetBalance) }

    func price(for assetId: String) -> Primitives.Price? {
        price(assetId: assetId).map { $0.toPrimitives().mapToPrice() }
    }

    func price(for assetId: Primitives.AssetId) -> Primitives.Price? {
        price(for: assetId.identifier)
    }

    var assetPrices: [Primitives.AssetId: Primitives.Price] {
        Dictionary(uniqueKeysWithValues: prices.map { (Primitives.AssetId(core: $0.assetId), $0.toPrimitives().mapToPrice()) })
    }
}
