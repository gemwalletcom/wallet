// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol Reconnectable: Sendable {
    func reconnection(attempt: UInt32, connectedFor duration: Duration) -> Reconnection
    func pingIntervalMilliseconds() -> UInt64
}
