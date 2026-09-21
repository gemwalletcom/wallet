// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import func Gemstone.formattedAmount
import struct Gemstone.GemFormattedNumber
@testable import GemstonePrimitives
import Testing

struct FormattedNumberTests {
    @Test
    func anAbbreviatedValueRoundsTheWayTheRecordAsks() {
        let toNearest = GemFormattedNumber(
            value: 1_235_999,
            unit: .currency(code: "USD"),
            display: .abbreviated,
            notation: .plain,
            tone: .plain,
            rounding: .toNearest,
        )
        let towardZero = GemFormattedNumber(
            value: 1_235_999,
            unit: .currency(code: "USD"),
            display: .abbreviated,
            notation: .plain,
            tone: .plain,
            rounding: .towardZero,
        )

        #expect(toNearest.text(locale: .US) == "$1.24M")
        #expect(towardZero.text(locale: .US) == "$1.23M", "the record asks for truncation and gets it")
    }

    @Test
    func anAmountReadsLikeTheValueFormatter() throws {
        let formatter = ValueFormatter(locale: .US, style: .auto)
        let values: [(BigInt, Int)] = [(5_205_516, 6), (99999, 6), (1992, 4), (1_239_999_000_000, 6), (546, 8)]

        for (value, decimals) in values {
            let number = try formattedAmount(value: formatter.double(from: value, decimals: decimals), symbol: "ATOM", style: .auto)
            #expect(number.text(locale: .US) == formatter.string(value, decimals: decimals, currency: "ATOM"))
        }
    }
}
