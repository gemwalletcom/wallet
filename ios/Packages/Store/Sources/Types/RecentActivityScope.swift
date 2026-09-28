// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum RecentActivityScope: Sendable {
    case wallet(WalletId)
    case allWallets
}
