// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct Reconnection: Sendable, Equatable {
    public let nextAttempt: Int
    public let delay: Duration

    public init(nextAttempt: Int, delay: Duration) {
        self.nextAttempt = nextAttempt
        self.delay = delay
    }
}
