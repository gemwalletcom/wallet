// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct DelegationBase: Codable, Equatable, Hashable, Sendable {
    public let assetId: AssetId
    public let state: DelegationState
    public let balance: BigInt
    public let shares: BigInt
    public let rewards: BigInt
    public let completionDate: Date?
    public let delegationId: String
    public let validatorId: String

    public init(
        assetId: AssetId,
        state: DelegationState,
        balance: BigInt,
        shares: BigInt,
        rewards: BigInt,
        completionDate: Date?,
        delegationId: String,
        validatorId: String,
    ) {
        self.assetId = assetId
        self.state = state
        self.balance = balance
        self.shares = shares
        self.rewards = rewards
        self.completionDate = completionDate
        self.delegationId = delegationId
        self.validatorId = validatorId
    }
}

public struct Delegation: Codable, Equatable, Hashable, Sendable {
    public let base: DelegationBase
    public let validator: DelegationValidator
    public let price: Price?

    public init(base: DelegationBase, validator: DelegationValidator, price: Price?) {
        self.base = base
        self.validator = validator
        self.price = price
    }
}
