// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationAction

public enum DelegationActionType: Hashable, Identifiable {
    public var id: Self {
        self
    }

    case stake, unstake, redelegate
    case deposit
    case withdraw
    case claimRewards

    init(_ action: GemDelegationAction) {
        self = switch action {
        case .stake: .stake
        case .unstake: .unstake
        case .redelegate: .redelegate
        case .withdraw: .withdraw
        case .deposit: .deposit
        }
    }
}
