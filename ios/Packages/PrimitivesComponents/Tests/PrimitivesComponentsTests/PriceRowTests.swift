// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemCurrencyStyle
import func Gemstone.priceRow
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
        #expect(priceText(-10) == nil, "a price below zero is no price")
        #expect(priceText(123_456) == "$123,456.00")
        #expect(priceText(10_123_456) == "$10,123,456.00")
    }

    @Test
    func aPriceNobodyQuotedIsNoPrice() {
        #expect(priceText(0) == nil)
        #expect(priceRow(price: nil, change: nil, currency: .usd, style: .short).price == nil)
    }

    @Test
    func theChangeCarriesItsSignAndTone() {
        let rising = priceRow(price: 10, change: 5, currency: .usd, style: .short).change
        let falling = priceRow(price: 10, change: -5, currency: .usd, style: .short).change

        #expect(rising?.text() == "+5.00%")
        #expect(rising?.tone.color == Colors.green)
        #expect(falling?.text() == "-5.00%")
        #expect(falling?.tone.color == Colors.red)
        #expect(priceRow(price: 10, change: nil, currency: .usd, style: .short).change == nil)
    }

    private func priceText(_ price: Double, style: GemCurrencyStyle = .short) -> String? {
        priceRow(price: price, change: nil, currency: .usd, style: style).price?.text()
    }
}
