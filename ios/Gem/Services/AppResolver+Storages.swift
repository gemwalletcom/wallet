// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let stores: Stores
        let systemPrompt = SystemPrompt()
        let keystorePassword: LocalKeystorePassword
        let keystore: LocalKeystore

        init() {
            stores = Stores(db: db)
            keystorePassword = LocalKeystorePassword(systemPrompt: systemPrompt)
            keystore = LocalKeystore(keystorePassword: keystorePassword)
        }
    }
}
