// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemDelegationAction
import enum Gemstone.GemStakeAction
import enum Gemstone.GemStakeSection
import Localization
import Primitives

extension GemStakeSection {
    var title: String {
        switch self {
        case .manage: Localized.Common.manage
        case .resources: Localized.Asset.resources
        case .delegations: Localized.Stake.delegations
        }
    }
}

extension GemStakeAction {
    var title: String {
        switch self {
        case .stake: Localized.Transfer.Stake.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        }
    }
}

extension GemDelegationAction {
    var title: String {
        switch self {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Transfer.Withdraw.title
        }
    }
}

extension StakeProviderType {
    var title: String {
        switch self {
        case .stake: Localized.Transfer.Stake.title
        case .earn: Localized.Common.earn
        }
    }
}
