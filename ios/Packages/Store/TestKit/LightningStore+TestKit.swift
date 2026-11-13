// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension LightningStore {
    static func mock(db: DB = .mock()) -> Self {
        LightningStore(db: db)
    }
}
