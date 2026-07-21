// Copyright (c). Gem Wallet. All rights reserved.

import ConnectivityService

public struct ConnectivityMonitorMock: ConnectivityMonitoring {
    private let stream: AsyncStream<ConnectivityState>

    public init(stream: AsyncStream<ConnectivityState> = AsyncStream { $0.finish() }) {
        self.stream = stream
    }

    public func stateStream() -> AsyncStream<ConnectivityState> {
        stream
    }
}

public extension ConnectivityService {
    static func mock(
        monitor: any ConnectivityMonitoring = ConnectivityMonitorMock(),
    ) -> ConnectivityService {
        ConnectivityService(monitor: monitor)
    }
}
