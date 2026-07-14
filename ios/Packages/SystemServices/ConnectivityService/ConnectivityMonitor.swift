// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Network

public protocol ConnectivityMonitoring: Sendable {
    func stateStream() -> AsyncStream<ConnectivityState>
}

public struct ConnectivityMonitor: ConnectivityMonitoring {
    private let queue = DispatchQueue(label: "com.gemwallet.connectivity")
    private let mapper = NWPathMapper()

    public init() {}

    public func stateStream() -> AsyncStream<ConnectivityState> {
        AsyncStream { continuation in
            let monitor = NWPathMonitor()
            monitor.pathUpdateHandler = { continuation.yield(mapper.state(from: $0)) }
            monitor.start(queue: queue)
            continuation.onTermination = { _ in monitor.cancel() }
        }
    }
}
