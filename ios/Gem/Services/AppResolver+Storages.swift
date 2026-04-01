// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Preferences
import Store

extension AppResolver {
    struct Storages {
        let db: DB = .init()
        let observablePreferences: ObservablePreferences = .default
        let keystore: any Keystore = LocalKeystore()
    }
}
