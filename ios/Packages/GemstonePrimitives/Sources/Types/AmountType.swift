// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemPerpetualPositionAction
import Primitives

public enum AmountStakeType: Equatable, Hashable, Sendable {
    case stake(validators: [DelegationValidator], recommended: DelegationValidator?)
    case unstake(Delegation)
    case redelegate(Delegation, validators: [DelegationValidator], recommended: DelegationValidator?)
    case withdraw(Delegation)
    case claimRewards(delegations: [Delegation])
    case freeze(Resource)
    case unfreeze(Resource)
}

public enum AmountType: Equatable, Hashable, Sendable {
    case transfer(recipient: GemPaymentRecipient)
    case deposit
    case withdraw
    case stake(AmountStakeType)
    case perpetual(GemPerpetualPositionAction)
    case earn(EarnType)
}
