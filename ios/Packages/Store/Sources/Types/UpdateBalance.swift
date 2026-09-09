// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct UpdateBalance {
    public let assetId: AssetId
    public let available: UpdateBalanceValue
    public let frozen: UpdateBalanceValue
    public let locked: UpdateBalanceValue
    public let staked: UpdateBalanceValue
    public let pending: UpdateBalanceValue
    public let pendingUnconfirmed: UpdateBalanceValue
    public let rewards: UpdateBalanceValue
    public let reserved: UpdateBalanceValue
    public let withdrawable: UpdateBalanceValue
    public let earn: UpdateBalanceValue
    public let metadata: BalanceMetadata?
    public let updatedAt: Date
    public let isActive: Bool

    public init(
        assetId: AssetId,
        available: UpdateBalanceValue = .zero,
        frozen: UpdateBalanceValue = .zero,
        locked: UpdateBalanceValue = .zero,
        staked: UpdateBalanceValue = .zero,
        pending: UpdateBalanceValue = .zero,
        pendingUnconfirmed: UpdateBalanceValue = .zero,
        rewards: UpdateBalanceValue = .zero,
        reserved: UpdateBalanceValue = .zero,
        withdrawable: UpdateBalanceValue = .zero,
        earn: UpdateBalanceValue = .zero,
        metadata: BalanceMetadata? = nil,
        updatedAt: Date,
        isActive: Bool,
    ) {
        self.assetId = assetId
        self.available = available
        self.frozen = frozen
        self.locked = locked
        self.staked = staked
        self.pending = pending
        self.pendingUnconfirmed = pendingUnconfirmed
        self.rewards = rewards
        self.reserved = reserved
        self.withdrawable = withdrawable
        self.earn = earn
        self.metadata = metadata
        self.updatedAt = updatedAt
        self.isActive = isActive
    }
}
