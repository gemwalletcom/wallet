// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstoneServices
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let storeManager: StoreManager
        let keystore = LocalKeystore(transferService: Gemstone.GemTransferService())

        init() {
            storeManager = StoreManager(db: db)
        }
    }
}
