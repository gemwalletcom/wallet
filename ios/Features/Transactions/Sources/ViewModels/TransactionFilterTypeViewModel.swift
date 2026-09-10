// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionFilter
import Localization
import Primitives

struct TransactionFilterTypeViewModel {
    private let type: GemTransactionFilter

    init(type: GemTransactionFilter) {
        self.type = type
    }

    var title: String {
        switch type {
        case .transfers: Localized.Transfer.title
        case .smartContract: Localized.Transfer.SmartContract.title
        case .swaps: Localized.Wallet.swap
        case .stake: Localized.Transfer.Stake.title
        case .perpetuals: Localized.Perpetuals.title
        case .others: Localized.Transfer.Other.title
        }
    }
}
