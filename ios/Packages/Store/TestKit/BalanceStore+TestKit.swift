// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension BalanceStore {
    static func mock(db: DB = .mock()) -> Self {
        BalanceStore(db: db)
    }
}
