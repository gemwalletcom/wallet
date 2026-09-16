// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSwapProgressStep
import enum Gemstone.GemTransactionFilter
import enum Gemstone.GemTransactionParticipantRole
import Localization

extension GemSwapProgressStep {
    var tagTitle: String? {
        switch self {
        case .completed: Localized.Transaction.Status.completed
        case .pending: Localized.Transaction.Status.inprogress
        case .waiting: nil
        case .failed: Localized.Transaction.Status.failed
        case .reverted: Localized.Transaction.Status.reverted
        case .refunded: Localized.Transaction.Status.refunded
        }
    }
}

extension GemTransactionFilter {
    var title: String {
        switch self {
        case .transfers: Localized.Transfer.title
        case .smartContract: Localized.Transfer.SmartContract.title
        case .swaps: Localized.Wallet.swap
        case .stake: Localized.Wallet.stake
        case .perpetuals: Localized.Perpetuals.title
        case .others: Localized.Transfer.Other.title
        }
    }
}

extension GemTransactionParticipantRole {
    var title: String {
        switch self {
        case .sender: Localized.Transaction.sender
        case .recipient: Localized.Transaction.recipient
        case .contract: Localized.Asset.contract
        case .validator: Localized.Stake.validator
        case .provider: Localized.Common.provider
        }
    }
}
