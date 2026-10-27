// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol Reconnectable: Sendable {
    func reconnection(attempt: Int, connectedFor duration: Duration) -> Reconnection
    var pingInterval: Duration { get }
}
