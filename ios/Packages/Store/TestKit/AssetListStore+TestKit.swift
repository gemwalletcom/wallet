// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension AssetListStore {
    static func mock(db: DB = .mock()) -> Self {
        AssetListStore(db: db)
    }
}
