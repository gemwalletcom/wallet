// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol Reconnectable: Sendable {
    func reconnectDelayMilliseconds(attempt: UInt32) -> UInt64
    func pingIntervalMilliseconds() -> UInt64
}
