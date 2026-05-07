// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct JobConfiguration: Equatable, Hashable, Sendable {
    public let initialIntervalMs: UInt32
    public let maxIntervalMs: UInt32
    public let stepFactor: Float

    public init(initialIntervalMs: UInt32, maxIntervalMs: UInt32, stepFactor: Float) {
        self.initialIntervalMs = initialIntervalMs
        self.maxIntervalMs = maxIntervalMs
        self.stepFactor = stepFactor
    }

    public var initialInterval: Duration {
        .milliseconds(Int(initialIntervalMs))
    }

    public var maxInterval: Duration {
        .milliseconds(Int(maxIntervalMs))
    }
}
