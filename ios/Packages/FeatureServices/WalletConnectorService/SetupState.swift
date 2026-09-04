// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor SetupState {
    private var isStarted = false

    func runOnce(_ operation: @Sendable () -> Void) {
        guard !isStarted else { return }
        operation()
        isStarted = true
    }
}
