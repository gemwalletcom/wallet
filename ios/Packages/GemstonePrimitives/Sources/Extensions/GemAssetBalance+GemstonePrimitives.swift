// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetBalance
import Primitives

public extension GemAssetBalance {
    init(_ balance: Balance, assetId: AssetId) {
        self.init(
            assetId: assetId.identifier,
            available: balance.available.description,
            frozen: balance.frozen.description,
            locked: balance.locked.description,
            staked: balance.staked.description,
            pending: balance.pending.description,
            pendingUnconfirmed: balance.pendingUnconfirmed.description,
            rewards: balance.rewards.description,
            reserved: balance.reserved.description,
            withdrawable: balance.withdrawable.description,
            earn: balance.earn.description,
            metadata: balance.metadata?.json(),
        )
    }
}
