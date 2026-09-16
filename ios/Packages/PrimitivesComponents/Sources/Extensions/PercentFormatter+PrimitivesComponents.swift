// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import enum Gemstone.GemPercentageStyle

public extension PercentFormatter {
    static let signed = PercentFormatter(style: .signed)
    static let unsigned = PercentFormatter(style: .unsigned)

    init(style: GemPercentageStyle, locale: Locale = .current) {
        let format = style.format()
        let fraction: ClosedRange<Int> = switch format.precision {
        case let .fraction(min, max): Int(min) ... Int(max)
        case let .significant(max): 0 ... Int(max)
        }
        self.init(locale: locale, fractionLength: fraction, showsSign: format.showsSign)
    }
}
