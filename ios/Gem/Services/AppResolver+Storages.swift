// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Preferences
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let storeManager: StoreManager
        let preferences: Preferences = .standard
        let keystore = LocalKeystore()

        init() {
            storeManager = StoreManager(db: db)
        }
    }
}
