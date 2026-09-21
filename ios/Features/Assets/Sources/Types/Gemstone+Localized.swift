// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives

extension StakeProviderType {
    var title: String {
        switch self {
        case .stake: Localized.Wallet.stake
        case .earn: Localized.Common.earn
        }
    }
}
