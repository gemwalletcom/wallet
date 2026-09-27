// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import func Gemstone.formattedAmount
import func Gemstone.formattedCurrency
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
            exact: nil,
        )
        let towardZero = GemFormattedNumber(
            value: 1_235_999,
            unit: .currency(code: "USD"),
            display: .abbreviated,
            notation: .plain,
            tone: .plain,
            rounding: .towardZero,
            exact: nil,
        )

        #expect(toNearest.text(locale: .US) == "$1.24M")
        #expect(towardZero.text(locale: .US) == "$1.23M", "the record asks for truncation and gets it")
    }

    @Test
    func aFullAmountReadsItsExactDigits() {
        let exact = "123.456789012345678901"
        let spent = GemFormattedNumber(
            value: -123.456789012345678901,
            unit: .symbol(symbol: "ETH"),
            display: .number(precision: .fraction(min: 0, max: 32)),
            notation: .signed,
            tone: .negative,
            rounding: .towardZero,
            exact: exact,
        )

        #expect(spent.text(locale: .US) == "-123.456789012345678901 ETH", "every digit past what a double holds")
        #expect(
            GemFormattedNumber(value: 0.5, unit: .symbol(symbol: "SOL"), display: .number(precision: .fraction(min: 0, max: 32)), notation: .plain, tone: .plain, rounding: .towardZero, exact: "0.5")
                .text(locale: Locale(identifier: "de_DE")) == "0,5 SOL",
            "the digits follow the reader's separator",
        )
    }

    @Test
    func anAbbreviatedValueKeepsItsSign() {
        let incoming = GemFormattedNumber(
            value: 1_235_999,
            unit: .currency(code: "USD"),
            display: .abbreviated,
            notation: .signed,
            tone: .positive,
            rounding: .toNearest,
            exact: nil,
        )

        #expect(incoming.text(locale: .US) == "+$1.24M")
        #expect(GemFormattedNumber(value: -1_235_999, unit: .currency(code: "USD"), display: .abbreviated, notation: .signed, tone: .plain, rounding: .toNearest, exact: nil).text(locale: .US) == "-$1.24M")
        #expect(GemFormattedNumber(value: -1_235_999, unit: .currency(code: "USD"), display: .abbreviated, notation: .plain, tone: .plain, rounding: .toNearest, exact: nil).text(locale: .US) == "-$1.24M")
    }

    @Test
    func aShortPriceReadsDustBelowTheThreshold() {
        let prices: [(Double, String)] = [(0.00000783, "<$0.0001"), (0.0001, "$0.0001"), (0.0345, "$0.0345"), (1234.5, "$1,234.50")]

        for (value, text) in prices {
            #expect(formattedCurrency(value: value, code: "USD", style: .short).text(locale: .US) == text)
        }
    }

    @Test
    func anAmountReadsWithTheAutoStyle() throws {
        let amounts: [(Double, String)] = [(5.205516, "5.2 ATOM"), (0.099999, "0.09999 ATOM"), (0.1992, "0.1992 ATOM"), (1_239_999, "1,239,999 ATOM"), (0.00000546, "0.00000546 ATOM")]

        for (value, text) in amounts {
            let formatted = try formattedAmount(value: value, symbol: "ATOM", style: .auto).text(locale: .US)
            #expect(formatted == text)
        }
    }
}
