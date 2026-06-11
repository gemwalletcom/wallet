// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

struct UnlockAttempt {
    let context: LAContext
    let task: Task<Void, Never>
}
