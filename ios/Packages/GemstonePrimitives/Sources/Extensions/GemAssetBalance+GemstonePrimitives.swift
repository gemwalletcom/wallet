// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemAssetBalance
import Primitives

public extension GemAssetBalance {
    init(_ balance: Balance, assetId: AssetId) {
        self.init(
            assetId: assetId.identifier,
            available: BigUInt(balance.available),
            frozen: BigUInt(balance.frozen),
            locked: BigUInt(balance.locked),
            staked: BigUInt(balance.staked),
            pending: BigUInt(balance.pending),
            pendingUnconfirmed: BigUInt(balance.pendingUnconfirmed),
            rewards: BigUInt(balance.rewards),
            reserved: BigUInt(balance.reserved),
            withdrawable: BigUInt(balance.withdrawable),
            earn: BigUInt(balance.earn),
            metadata: balance.metadata?.map(),
        )
    }
}
