// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing

@testable import Primitives

struct JobConfigurationTests {
    @Test
    func nextInterval() {
        let config = JobConfiguration(
            initialInterval: .seconds(5),
            maxInterval: .seconds(10),
            stepFactor: 1.5,
        )

        #expect(config.nextInterval(after: .seconds(5)) == .seconds(7.5))
        #expect(config.nextInterval(after: .seconds(7)) == .seconds(10))
        #expect(config.nextInterval(after: .seconds(1)) == .seconds(5))

        let chained = config.nextInterval(after: config.nextInterval(after: .seconds(5)))
        #expect(chained == .seconds(10))
    }

    @Test
    func initialIntervalCappedAtMax() {
        let clamped = JobConfiguration(
            initialInterval: .seconds(50),
            maxInterval: .seconds(10),
            stepFactor: 1.5,
        )
        #expect(clamped.initialInterval == .seconds(10))
        #expect(clamped.nextInterval(after: .seconds(2)) == .seconds(10))
    }
}
