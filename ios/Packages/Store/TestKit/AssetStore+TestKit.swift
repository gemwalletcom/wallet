// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public extension AssetStore {
    static func mock(db: DB = .mock()) -> Self {
        AssetStore(db: db)
    }
}
