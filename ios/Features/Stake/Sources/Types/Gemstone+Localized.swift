// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.DelegationState
import enum Gemstone.GemDelegationAction
import enum Gemstone.GemDelegationCompletion
import enum Gemstone.GemDelegationRow
import enum Gemstone.GemStakeAction
import enum Gemstone.GemStakeInfoRow
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

extension GemStakeInfoRow {
    var title: String {
        switch self {
        case .apr: Localized.Stake.apr("")
        case .lockTime: Localized.Stake.lockTime
        case .minimumAmount: Localized.Stake.minimumAmount
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

extension GemDelegationCompletion {
    var title: String {
        switch self {
        case .activeIn: Localized.Stake.activeIn
        case .availableIn: Localized.Stake.availableIn
        }
    }
}

func delegationRowTitle(_ row: GemDelegationRow, providerType: StakeProviderType, completion: GemDelegationCompletion?) -> String {
    switch row {
    case .provider: providerType.providerTitle
    case .apr: Localized.Stake.apr("")
    case .status: Localized.Transaction.status
    case .completionDate: completion?.title ?? .empty
    case .rewards: Localized.Stake.rewards
    }
}

extension StakeProviderType {
    var title: String {
        switch self {
        case .stake: Localized.Transfer.Stake.title
        case .earn: Localized.Common.earn
        }
    }

    var providerTitle: String {
        switch self {
        case .stake: Localized.Stake.validator
        case .earn: Localized.Common.provider
        }
    }
}

extension Gemstone.DelegationState {
    var title: String {
        switch self {
        case .active: Localized.Stake.active
        case .pending: Localized.Stake.pending
        case .inactive: Localized.Stake.inactive
        case .activating: Localized.Stake.activating
        case .deactivating: Localized.Stake.deactivating
        case .awaitingWithdrawal: Localized.Stake.awaitingWithdrawal
        }
    }
}
