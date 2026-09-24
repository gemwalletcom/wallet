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
