// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PlatformStore {
    static var current: PlatformStore {
        #if targetEnvironment(simulator)
            .local
        #else
            .appStore
        #endif
    }
}
