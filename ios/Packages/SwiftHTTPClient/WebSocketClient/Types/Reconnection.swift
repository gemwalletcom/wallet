// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct Reconnection: Sendable, Equatable {
    public let nextAttempt: UInt32
    public let delay: Duration

    public init(nextAttempt: UInt32, delay: Duration) {
        self.nextAttempt = nextAttempt
        self.delay = delay
    }
}
