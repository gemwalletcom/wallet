// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSearchScope
import Primitives

public extension WalletSearchTag {
    var gemScope: GemSearchScope {
        switch self {
        case .all: .all
        case let .list(id): .list(id: id)
        }
    }
}
