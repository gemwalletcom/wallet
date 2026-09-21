// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.priceRow
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import Style
import Testing

struct PriceRowTests {
    @Test
    func aQuotedPriceKeepsItsSmallDigits() {
        #expect(priceText(1.2) == "$1.20")
        #expect(priceText(10) == "$10.00")
        #expect(priceText(0.000000123) == "$0.000000123")
        #expect(priceText(0.00000000123) == "$0.00000000123")
        #expect(priceText(0.000000123456) == "$0.0000001235")
        #expect(priceText(-10) == nil, "a price below zero is no price")
        #expect(priceText(123_456) == "$123,456.00")
        #expect(priceText(10_123_456) == "$10,123,456.00")
    }

    @Test
    func aPriceNobodyQuotedIsNoPrice() {
        #expect(priceText(0) == nil)
        #expect(priceRow(price: nil, change: nil, currency: .usd).price == nil)
    }

    @Test
    func theChangeCarriesItsSignAndTone() {
        let rising = priceRow(price: 10, change: 5, currency: .usd).change
        let falling = priceRow(price: 10, change: -5, currency: .usd).change

        #expect(rising?.text() == "+5.00%")
        #expect(rising?.tone.color == Colors.green)
        #expect(falling?.text() == "-5.00%")
        #expect(falling?.tone.color == Colors.red)
        #expect(priceRow(price: 10, change: nil, currency: .usd).change == nil)
    }

    private func priceText(_ price: Double) -> String? {
        priceRow(price: price, change: nil, currency: .usd).price?.text()
    }
}
