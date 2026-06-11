// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

struct UnlockAttempt {
    let context: LAContext
    let task: Task<Void, Never>
    var isInvalidated = false

    func invalidated() -> UnlockAttempt {
        context.invalidate()
        return UnlockAttempt(context: context, task: task, isInvalidated: true)
    }
}
