// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.Delegation
import struct Gemstone.DelegationBase
import enum Gemstone.EarnType
import struct Gemstone.RedelegateData
import enum Gemstone.StakeType
import Primitives

public extension Primitives.DelegationBase {
    init(core: Gemstone.DelegationBase) {
        self.init(
            assetId: Primitives.AssetId(core: core.assetId),
            state: core.state.map(),
            balance: BigInt(core.balance),
            shares: BigInt(core.shares),
            rewards: BigInt(core.rewards),
            completionDate: core.completionDate,
            delegationId: core.delegationId,
            validatorId: core.validatorId,
        )
    }

    func map() -> Gemstone.DelegationBase {
        Gemstone.DelegationBase(
            assetId: assetId.identifier,
            state: state.map(),
            balance: balance.magnitude,
            shares: shares.magnitude,
            rewards: rewards.magnitude,
            completionDate: completionDate,
            delegationId: delegationId,
            validatorId: validatorId,
        )
    }
}

public extension Primitives.Delegation {
    init(core: Gemstone.Delegation) {
        self.init(
            base: Primitives.DelegationBase(core: core.base),
            validator: core.validator.map(),
            price: .none,
        )
    }

    func map() -> Gemstone.Delegation {
        Gemstone.Delegation(base: base.map(), validator: validator.map())
    }
}

public extension Primitives.RedelegateData {
    init(core: Gemstone.RedelegateData) {
        self.init(
            delegation: Primitives.Delegation(core: core.delegation),
            toValidator: core.toValidator.map(),
        )
    }

    func map() -> Gemstone.RedelegateData {
        Gemstone.RedelegateData(delegation: delegation.map(), toValidator: toValidator.map())
    }
}

public extension Primitives.StakeType {
    init(core: Gemstone.StakeType) {
        self = switch core {
        case let .stake(validator): .stake(validator.map())
        case let .unstake(delegation): .unstake(Primitives.Delegation(core: delegation))
        case let .redelegate(data): .redelegate(Primitives.RedelegateData(core: data))
        case let .rewards(validators): .rewards(validators.map { $0.map() })
        case let .withdraw(delegation): .withdraw(Primitives.Delegation(core: delegation))
        case let .freeze(resource): .freeze(resource.map())
        case let .unfreeze(resource): .unfreeze(resource.map())
        }
    }

    func map() -> Gemstone.StakeType {
        switch self {
        case let .stake(validator): .stake(validator.map())
        case let .unstake(delegation): .unstake(delegation.map())
        case let .redelegate(data): .redelegate(data.map())
        case let .rewards(validators): .rewards(validators.map { $0.map() })
        case let .withdraw(delegation): .withdraw(delegation.map())
        case let .freeze(resource): .freeze(resource.map())
        case let .unfreeze(resource): .unfreeze(resource.map())
        }
    }
}

public extension Primitives.EarnType {
    init(core: Gemstone.EarnType) {
        self = switch core {
        case let .deposit(validator): .deposit(validator.map())
        case let .withdraw(delegation): .withdraw(Primitives.Delegation(core: delegation))
        }
    }

    func map() -> Gemstone.EarnType {
        switch self {
        case let .deposit(validator): .deposit(validator.map())
        case let .withdraw(delegation): .withdraw(delegation.map())
        }
    }
}
