// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public final class ConnectionComponentHealth: ConnectionComponentMonitoring {
    public let component: ConnectionComponent
    private let continuations = Locked<[UUID: AsyncStream<Bool>.Continuation]>(wrappedValue: [:])
    private let isHealthy = Locked<Bool?>(wrappedValue: nil)

    public init(component: ConnectionComponent) {
        self.component = component
    }

    public func report(isHealthy: Bool) {
        self.isHealthy.wrappedValue = isHealthy
        for continuation in continuations.wrappedValue.values {
            continuation.yield(isHealthy)
        }
    }

    public func healthStream() -> AsyncStream<Bool> {
        let (stream, continuation) = AsyncStream<Bool>.makeStream()
        let id = UUID()
        continuations.wrappedValue[id] = continuation
        continuation.onTermination = { [continuations] _ in
            continuations.wrappedValue[id] = nil
        }
        if let isHealthy = isHealthy.wrappedValue {
            continuation.yield(isHealthy)
        }
        return stream
    }
}
