// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAssetPrice
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
            asset: Primitives.Asset(asset),
            balance: Primitives.Balance(balance),
            price: price.map { Primitives.Price(price: $0.price, priceChangePercentage24h: $0.priceChangePercentage24h, updatedAt: Date(timeIntervalSince1970: TimeInterval($0.updatedAt))) }
        )
    }
}
