// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemConnectionService
import WebSocketClient

extension GemConnectionService: @retroactive Reconnectable {
    public func reconnection(attempt: UInt32, connectedFor duration: Duration) -> Reconnection {
        let (seconds, attoseconds) = duration.components
        let next = reconnection(attempt: attempt, connected: TimeInterval(seconds) + TimeInterval(attoseconds) / 1e18)
        return Reconnection(nextAttempt: next.nextAttempt, delay: .seconds(next.delay))
    }
}
