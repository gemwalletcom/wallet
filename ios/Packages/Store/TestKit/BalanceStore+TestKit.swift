// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension BalanceStore {
    static func mock(db: DB = .mock()) -> Self {
        BalanceStore(db: db)
    }
}

public extension AssetConfiguration {
    static let disabled = AssetConfiguration(isEnabled: false, isPinned: false)

    static func pinned(_ isPinned: Bool) -> AssetConfiguration {
        AssetConfiguration(isEnabled: nil, isPinned: isPinned)
    }
}
