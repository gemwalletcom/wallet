// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension ConnectionStore {
    static func mock(
        db: DB = .mock(),
    ) -> ConnectionStore {
        ConnectionStore(db: db)
    }
}
