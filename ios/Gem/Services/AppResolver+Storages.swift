// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let stores: Stores

        init() {
            stores = Stores(db: db)
        }
    }
}
