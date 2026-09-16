// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let stores: Stores
        let keystore = LocalKeystore()

        init() {
            stores = Stores(db: db)
        }
    }
}
