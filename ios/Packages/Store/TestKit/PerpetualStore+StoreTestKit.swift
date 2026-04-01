// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension PerpetualStore {
    static func mock() -> Self {
        Self(db: DB.mock())
    }
}
