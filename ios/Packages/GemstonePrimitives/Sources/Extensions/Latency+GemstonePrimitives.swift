// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

extension Primitives.Latency {
    static func from(milliseconds: Double) -> Primitives.Latency {
        Primitives.Latency(
            latency_type: LatencyType.from(duration: milliseconds),
            value: milliseconds,
        )
    }
}
