// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor SetupState {
    private var isSetup = false

    func start(_ operation: @Sendable () -> Void) {
        guard !isSetup else { return }
        operation()
        isSetup = true
    }
}
