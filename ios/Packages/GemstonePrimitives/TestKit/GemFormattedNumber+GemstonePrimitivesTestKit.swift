// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemNumberDisplay
import enum Gemstone.GemNumberNotation
import enum Gemstone.GemNumberRounding
import enum Gemstone.GemNumberUnit
import enum Gemstone.GemPrecision
import enum Gemstone.GemValueTone

public extension GemFormattedNumber {
    static func mock(
        value: Double = 1,
        unit: GemNumberUnit = .currency(code: "USD"),
        display: GemNumberDisplay = .number(precision: .fraction(min: 2, max: 2)),
        notation: GemNumberNotation = .signed,
        tone: GemValueTone = .plain,
        rounding: GemNumberRounding = .toNearest,
        exact: String? = nil,
    ) -> GemFormattedNumber {
        GemFormattedNumber(value: value, unit: unit, display: display, notation: notation, tone: tone, rounding: rounding, exact: exact)
    }
}
