// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

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
    case transfer(recipient: RecipientData)
    case deposit(recipient: RecipientData)
    case withdraw(recipient: RecipientData)
    case stake(AmountStakeType)
    case perpetual(PerpetualRecipientData)
    case earn(EarnType)
}
