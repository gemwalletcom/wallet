// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol ConnectionComponentMonitoring: Sendable {
    var component: ConnectionComponent { get }
    func healthStream() -> AsyncStream<Bool>
}
