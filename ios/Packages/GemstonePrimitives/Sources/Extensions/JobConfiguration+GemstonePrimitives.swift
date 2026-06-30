// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemJobConfiguration
import Primitives

public extension GemJobConfiguration {
    init(_ config: JobConfiguration) {
        self.init(
            initialIntervalMs: config.initialIntervalMs,
            maxIntervalMs: config.maxIntervalMs,
            stepFactor: config.stepFactor,
        )
    }

    func map() -> JobConfiguration {
        JobConfiguration(
            initialIntervalMs: initialIntervalMs,
            maxIntervalMs: maxIntervalMs,
            stepFactor: stepFactor,
        )
    }
}

public extension JobConfiguration {
    func nextInterval(after currentIntervalMs: UInt32) -> UInt32 {
        GemJobConfiguration(self).nextIntervalMs(currentIntervalMs: currentIntervalMs)
    }
}
