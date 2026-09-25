// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WebSocketClient

public struct ReconnectableMock: Reconnectable {
    private let delayMilliseconds: UInt64

    public init(delayMilliseconds: UInt64 = 0) {
        self.delayMilliseconds = delayMilliseconds
    }

    public func reconnection(attempt: UInt32, connectedFor _: Duration) -> Reconnection {
        Reconnection(nextAttempt: attempt + 1, delay: .milliseconds(delayMilliseconds))
    }

    public var pingInterval: Duration {
        .zero
    }
}
