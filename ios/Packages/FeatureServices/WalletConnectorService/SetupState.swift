// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

actor SetupState {
    private var isSetup = false

    func start(_ operation: @Sendable () throws -> Void) rethrows {
        guard !isSetup else { return }
        try operation()
        isSetup = true
    }
}
