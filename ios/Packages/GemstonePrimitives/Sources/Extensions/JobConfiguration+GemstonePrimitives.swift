// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemJobConfiguration
import Primitives

public extension GemJobConfiguration {
    func map() -> JobConfiguration {
        JobConfiguration(
            initialIntervalMs: initialIntervalMs,
            maxIntervalMs: maxIntervalMs,
            stepFactor: stepFactor,
        )
    }
}
