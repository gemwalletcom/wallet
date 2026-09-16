// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemBalanceRow
import Localization
import Primitives

extension GemBalanceRow {
    func title(stakeProvider: StakeProviderType) -> String {
        switch self {
        case .available: Localized.Asset.Balances.available
        case .staked, .earn: stakeProvider.title
        case .pendingUnconfirmed: Localized.Stake.pending
        case .reserved: Localized.Asset.Balances.reserved
        }
    }
}

extension StakeProviderType {
    var title: String {
        switch self {
        case .stake: Localized.Wallet.stake
        case .earn: Localized.Common.earn
        }
    }
}
