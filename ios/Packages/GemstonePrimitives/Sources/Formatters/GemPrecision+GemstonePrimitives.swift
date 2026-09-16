// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPrecision

public extension GemPrecision {
    var formatStyle: NumberFormatStyleConfiguration.Precision {
        switch self {
        case let .fraction(min, max): .fractionLength(Int(min) ... Int(max))
        case let .significant(max): .significantDigits(1 ... Int(max))
        }
    }
}
