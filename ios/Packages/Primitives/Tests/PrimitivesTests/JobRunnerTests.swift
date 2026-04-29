// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing

@testable import Primitives

struct JobRunnerTests {
    @Test
    func nextInterval() {
        let config = JobConfiguration(initialIntervalMs: 5_000, maxIntervalMs: 10_000, stepFactor: 1.5)

        #expect(JobRunner.getNextInterval(after: .seconds(5), config: config) == .seconds(7.5))
        #expect(JobRunner.getNextInterval(after: .seconds(7), config: config) == .seconds(10))
        #expect(JobRunner.getNextInterval(after: .seconds(1), config: config) == .seconds(5))

        let chained = JobRunner.getNextInterval(after: JobRunner.getNextInterval(after: .seconds(5), config: config), config: config)
        #expect(chained == .seconds(10))
    }
}
