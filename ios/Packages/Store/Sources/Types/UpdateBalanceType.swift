// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum UpdateBalanceType {
    case coin(UpdateCoinBalance)
    case token(UpdateTokenBalance)
    case stake(UpdateStakeBalance)
    case perpetual(UpdatePerpetualBalance)
    case earn(UpdateEarnBalance)

    var metadata: BalanceMetadata? {
        switch self {
        case let .stake(balance): balance.metadata
        case .coin, .perpetual, .token, .earn: .none
        }
    }
}

public struct UpdateCoinBalance {
    public let available: UpdateBalanceValue
    public let reserved: UpdateBalanceValue
    public let pendingUnconfirmed: UpdateBalanceValue

    public init(
        available: UpdateBalanceValue,
        reserved: UpdateBalanceValue,
        pendingUnconfirmed: UpdateBalanceValue = .zero,
    ) {
        self.available = available
        self.reserved = reserved
        self.pendingUnconfirmed = pendingUnconfirmed
    }
}

public struct UpdateTokenBalance {
    public let available: UpdateBalanceValue

    public init(available: UpdateBalanceValue) {
        self.available = available
    }
}

public struct UpdateStakeBalance {
    public let staked: UpdateBalanceValue
    public let pending: UpdateBalanceValue
    public let frozen: UpdateBalanceValue
    public let locked: UpdateBalanceValue
    public let rewards: UpdateBalanceValue
    public let metadata: BalanceMetadata?

    public init(
        staked: UpdateBalanceValue,
        pending: UpdateBalanceValue,
        frozen: UpdateBalanceValue,
        locked: UpdateBalanceValue,
        rewards: UpdateBalanceValue,
        metadata: BalanceMetadata? = nil,
    ) {
        self.staked = staked
        self.pending = pending
        self.frozen = frozen
        self.locked = locked
        self.rewards = rewards
        self.metadata = metadata
    }
}

public struct UpdatePerpetualBalance {
    public let available: UpdateBalanceValue
    public let reserved: UpdateBalanceValue
    public let withdrawable: UpdateBalanceValue

    public init(
        available: UpdateBalanceValue,
        reserved: UpdateBalanceValue,
        withdrawable: UpdateBalanceValue,
    ) {
        self.available = available
        self.reserved = reserved
        self.withdrawable = withdrawable
    }
}

public struct UpdateEarnBalance {
    public let balance: UpdateBalanceValue

    public init(balance: UpdateBalanceValue) {
        self.balance = balance
    }
}

extension UpdateBalanceValue: CustomStringConvertible {
    public var description: String {
        value
    }
}

extension UpdateBalanceType: CustomStringConvertible {
    public var description: String {
        switch self {
        case let .coin(balance): balance.description
        case let .token(balance): balance.description
        case let .stake(balance): balance.description
        case let .perpetual(balance): balance.description
        case let .earn(balance): balance.description
        }
    }
}

extension UpdateCoinBalance: CustomStringConvertible {
    public var description: String {
        "coin(available: \(available), reserved: \(reserved), pendingUnconfirmed: \(pendingUnconfirmed))"
    }
}

extension UpdateTokenBalance: CustomStringConvertible {
    public var description: String {
        "token(available: \(available))"
    }
}

extension UpdateStakeBalance: CustomStringConvertible {
    public var description: String {
        "stake(staked: \(staked), pending: \(pending), frozen: \(frozen), locked: \(locked), rewards: \(rewards))"
    }
}

extension UpdatePerpetualBalance: CustomStringConvertible {
    public var description: String {
        "perpetual(available: \(available), reserved: \(reserved), withdrawable: \(withdrawable))"
    }
}

extension UpdateEarnBalance: CustomStringConvertible {
    public var description: String {
        "earn(balance: \(balance))"
    }
}
