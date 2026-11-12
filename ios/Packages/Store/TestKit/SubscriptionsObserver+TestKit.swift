// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension SubscriptionsObserver {
    static func mock(
        db: DB = DB.mock(),
    ) -> SubscriptionsObserver {
        SubscriptionsObserver(db: db.dbPool)
    }
}
