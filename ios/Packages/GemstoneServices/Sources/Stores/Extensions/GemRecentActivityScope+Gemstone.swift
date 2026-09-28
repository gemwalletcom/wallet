// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemRecentActivityScope
import Store

public extension GemRecentActivityScope {
    func map() -> RecentActivityScope {
        switch self {
        case let .wallet(walletId): .wallet(walletId)
        case .allWallets: .allWallets
        }
    }
}
