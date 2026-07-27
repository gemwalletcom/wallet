// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public struct PriceChangeCalculator: Sendable {
    private let calculator = Gemstone.PriceChangeCalculator()

    public init() {}

    public func percentage(from: Double, to: Double) -> Double {
        calculator.percentage(from: from, to: to)
    }

    public func amount(percentage: Double, value: Double) -> Double {
        calculator.amount(percentage: percentage, value: value)
    }
}
