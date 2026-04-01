// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing

@testable import Primitives

@MainActor
struct RetryIntervalCalculatorTests {
    let fixedConfig = JobConfiguration.fixed(duration: .seconds(5))

    let adaptiveConfig = JobConfiguration.adaptive(
        configuration: AdaptiveConfiguration(
            initialInterval: .seconds(5),
            maxInterval: .seconds(10),
            stepFactor: 1.5,
        ),
    )
    @Test
    func fixedAlwaysReturnsSameInterval() async {
        let result1 = RetryIntervalCalculator.nextInterval(
            config: fixedConfig,
            currentInterval: .seconds(2),
        )
        let result2 = RetryIntervalCalculator.nextInterval(
            config: fixedConfig,
            currentInterval: .seconds(30),
        )
        #expect(result1 == .seconds(5))
        #expect(result2 == .seconds(5))
    }

    @Test
    func adaptiveStepFactorGrowth() async {
        let result = RetryIntervalCalculator.nextInterval(
            config: adaptiveConfig,
            currentInterval: .seconds(5),
        )
        #expect(result == .seconds(7.5))
    }

    @Test
    func adaptiveClampsToMax() async {
        let result = RetryIntervalCalculator.nextInterval(
            config: adaptiveConfig,
            currentInterval: .seconds(7),
        )
        #expect(result == .seconds(10))
    }

    @Test
    func adaptiveTwoStepGrowth() async {
        let first = RetryIntervalCalculator.nextInterval(
            config: adaptiveConfig,
            currentInterval: .seconds(5),
        ) // 7.5
        let second = RetryIntervalCalculator.nextInterval(
            config: adaptiveConfig,
            currentInterval: first,
        ) // 7.5 × 1.5 = 11.25 → 10
        #expect(first == .seconds(7.5))
        #expect(second == .seconds(10))
    }

    @Test
    func initialIntervalNotHigherThenMaxInternal() async {
        let adaptiveConfig = JobConfiguration.adaptive(
            configuration: AdaptiveConfiguration(
                initialInterval: .seconds(50),
                maxInterval: .seconds(10),
                stepFactor: 1.5,
            ),
        )
        let result1 = RetryIntervalCalculator.nextInterval(
            config: adaptiveConfig,
            currentInterval: .seconds(2),
        )
        #expect(result1 == .seconds(10))
    }
}
