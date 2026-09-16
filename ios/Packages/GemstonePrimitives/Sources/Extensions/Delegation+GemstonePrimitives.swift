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
            state: core.state.toPrimitives(),
            balance: BigInt(core.balance),
            shares: BigInt(core.shares),
            rewards: BigInt(core.rewards),
            completionDate: core.completionDate,
            delegationId: core.delegationId,
            validatorId: core.validatorId,
        )
    }

    func toGem() -> Gemstone.DelegationBase {
        Gemstone.DelegationBase(
            assetId: assetId.identifier,
            state: state.toGem(),
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
            validator: core.validator.toPrimitives(),
            price: core.price?.toPrimitives(),
        )
    }

    func toGem() -> Gemstone.Delegation {
        Gemstone.Delegation(base: base.toGem(), validator: validator.toGem(), price: price?.toGem())
    }
}

public extension Primitives.RedelegateData {
    init(core: Gemstone.RedelegateData) {
        self.init(
            delegation: Primitives.Delegation(core: core.delegation),
            toValidator: core.toValidator.toPrimitives(),
        )
    }

    func toGem() -> Gemstone.RedelegateData {
        Gemstone.RedelegateData(delegation: delegation.toGem(), toValidator: toValidator.toGem())
    }
}

public extension Primitives.StakeType {
    func toGem() -> Gemstone.StakeType {
        switch self {
        case let .stake(validator): .stake(validator.toGem())
        case let .unstake(delegation): .unstake(delegation.toGem())
        case let .redelegate(data): .redelegate(data.toGem())
        case let .rewards(validators): .rewards(validators.map { $0.toGem() })
        case let .withdraw(delegation): .withdraw(delegation.toGem())
        case let .freeze(resource): .freeze(resource.toGem())
        case let .unfreeze(resource): .unfreeze(resource.toGem())
        }
    }
}
