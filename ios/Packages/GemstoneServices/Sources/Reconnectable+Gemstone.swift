// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemConnectionService
import GemstonePrimitives
import Primitives
import WebSocketClient

extension GemConnectionService: @retroactive Reconnectable {
    public func reconnection(attempt: UInt32, connectedFor duration: Duration) -> Reconnection {
        let next = reconnection(attempt: attempt, connected: duration.timeInterval)
        return Reconnection(nextAttempt: next.nextAttempt, delay: .seconds(next.delay))
    }

    public var pingInterval: Duration {
        GemConstants.pingInterval
    }
}
