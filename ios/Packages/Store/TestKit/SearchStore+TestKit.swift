// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension SearchStore {
    static func mock(db: DB = .mock()) -> Self {
        SearchStore(db: db)
    }
}
