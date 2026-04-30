// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.JobConfiguration
import Primitives

public extension Gemstone.JobConfiguration {
    func map() -> Primitives.JobConfiguration {
        Primitives.JobConfiguration(
            initialIntervalMs: initialIntervalMs,
            maxIntervalMs: maxIntervalMs,
            stepFactor: stepFactor,
        )
    }
}
