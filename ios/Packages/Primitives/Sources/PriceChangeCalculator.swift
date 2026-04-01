// Copyright (c). Gem Wallet. All rights reserved.

public enum PriceChangeCalculatorType {
    case percentage(from: Double, to: Double)
    case amount(percentage: Double, value: Double)
}

public enum PriceChangeCalculator {
    public static func calculate(_ type: PriceChangeCalculatorType) -> Double {
        switch type {
        case let .percentage(from, to):
            return from == 0 ? 0 : (to - from) / from * 100
        case let .amount(percentage, value):
            let denominator = 100 + percentage
            return denominator == 0 ? 0 : value * percentage / denominator
        }
    }
}
