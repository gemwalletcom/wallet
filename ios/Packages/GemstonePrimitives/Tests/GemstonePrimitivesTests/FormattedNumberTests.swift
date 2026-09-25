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
    func aShortPriceReadsLikeTheCurrencyFormatter() {
        let formatter = CurrencyFormatter(type: .short, locale: .US, currencyCode: "USD")

        for value in [0.00000783, 0.0001, 0.0345, 1234.5] {
            #expect(formattedCurrency(value: value, code: "USD", style: .short).text(locale: .US) == formatter.string(value))
        }
        #expect(formattedCurrency(value: 0.00000783, code: "USD", style: .short).text(locale: .US) == "<$0.0001")
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
