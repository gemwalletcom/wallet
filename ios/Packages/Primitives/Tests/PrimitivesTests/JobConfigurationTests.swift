// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing

@testable import Primitives

struct JobConfigurationTests {
    @Test
    func nextInterval() {
        let config = JobConfiguration(initialIntervalMs: 5_000, maxIntervalMs: 10_000, stepFactor: 1.5)

        #expect(config.nextInterval(after: .seconds(5)) == .seconds(7.5))
        #expect(config.nextInterval(after: .seconds(7)) == .seconds(10))
        #expect(config.nextInterval(after: .seconds(1)) == .seconds(5))

        let chained = config.nextInterval(after: config.nextInterval(after: .seconds(5)))
        #expect(chained == .seconds(10))
    }
}
