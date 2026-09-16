// Copyright (c). Gem Wallet. All rights reserved.

import LocalAuthentication

struct UnlockAttempt {
    let number: UInt32
    let context: LAContext
    let task: Task<Void, Never>
}
