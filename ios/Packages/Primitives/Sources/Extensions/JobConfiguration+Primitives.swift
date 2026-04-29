// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension JobConfiguration {
    var initialInterval: Duration {
        .milliseconds(Int(initialIntervalMs))
    }

    var maxInterval: Duration {
        .milliseconds(Int(maxIntervalMs))
    }

    func nextInterval(after current: Duration) -> Duration {
        max(initialInterval, min(current * Double(stepFactor), maxInterval))
    }
}
