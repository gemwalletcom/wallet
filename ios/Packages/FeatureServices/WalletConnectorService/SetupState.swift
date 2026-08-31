// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor SetupState {
    private var isSetup = false

    func start() -> Bool {
        if isSetup {
            return false
        }
        isSetup = true
        return true
    }
}
