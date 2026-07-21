// Copyright (c). Gem Wallet. All rights reserved.

import ConnectivityService
import Primitives

public struct InternetConnectionMonitor: ConnectionComponentMonitoring {
    private let connectivity: ConnectivityService

    public init(connectivity: ConnectivityService = ConnectivityService()) {
        self.connectivity = connectivity
    }

    public var component: ConnectionComponent { .internet }

    public func healthStream() -> AsyncStream<Bool> {
        AsyncStream { continuation in
            let task = Task {
                await connectivity.start()
                for await state in await connectivity.observe() {
                    continuation.yield(!state.isOffline)
                }
                continuation.finish()
            }
            continuation.onTermination = { _ in task.cancel() }
        }
    }
}
