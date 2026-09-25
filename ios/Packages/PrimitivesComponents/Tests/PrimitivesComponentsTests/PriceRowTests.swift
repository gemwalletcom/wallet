// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.formattedCurrency
import func Gemstone.formattedPercentage
import enum Gemstone.GemCurrencyStyle
import enum Gemstone.GemValueTone
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import Style
import Testing

struct PriceRowTests {
    @Test
    func aQuotedPriceKeepsItsSmallDigitsWhereTheRowHasRoomForThem() {
        #expect(priceText(1.2) == "$1.20")
        #expect(priceText(10) == "$10.00")
        #expect(priceText(0.000123456) == "$0.0001235")
        #expect(priceText(0.0001) == "$0.0001")
        #expect(priceText(0.000000123) == "<$0.0001", "a list row reads a price that would wrap as dust, the way an amount does")
        #expect(priceText(0.00000000123) == "<$0.0001")
        #expect(priceText(0.000000123, style: .currency) == "$0.000000123", "a detail row has the width for the digits")
        #expect(priceText(123_456) == "$123,456.00")
        #expect(priceText(10_123_456) == "$10,123,456.00")
    }

    @Test
    func theChangeCarriesItsSignAndTone() {
        #expect(formattedPercentage(value: 5, style: .signed).text() == "+5.00%")
        #expect(formattedPercentage(value: -5, style: .signed).text() == "-5.00%")
        #expect(GemValueTone.positive.color == Colors.green)
        #expect(GemValueTone.negative.color == Colors.red)
    }

    private func priceText(_ price: Double, style: GemCurrencyStyle = .short) -> String {
        formattedCurrency(value: price, code: Currency.usd.rawValue, style: style).text()
    }
}
