// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct ChartHeaderTests {
    @Test
    func theValueReadsAsAPriceOrAsAChange() {
        #expect(GemChartData.mock(currency: .usd).headerAt(value: 100).value.text() == "$100.00")
        #expect(GemChartData.mock(valueType: .priceChange, base: 0, currency: .usd).headerAt(value: 100).value.text() == "+$100.00")
        #expect(GemChartData.mock(valueType: .priceChange, base: 0, currency: .usd).headerAt(value: -50).value.text() == "-$50.00")
    }

    @Test
    func onlyAChangeIsToned() {
        #expect(GemChartData.mock(currency: .usd).headerAt(value: 100).value.tone.color == Colors.black)
        #expect(GemChartData.mock(valueType: .priceChange, base: 0, currency: .usd).headerAt(value: 100).value.tone.color == Colors.green)
    }

    @Test
    func theChangeCarriesItsSignBracketsAndTone() {
        #expect(GemChartData.mock(base: 100, currency: .usd).headerAt(value: 105.5).change?.text() == "+5.50%")
        #expect(GemChartData.mock(base: 0, currency: .usd).headerAt(value: 0).change == nil)
        #expect(GemChartData.mock(valueType: .priceChange, base: 100, showsSecondaryValue: true, currency: .usd).headerAt(value: 110).change?.text() == "(10.00%)")
        #expect(GemChartData.mock(base: 100, currency: .usd).headerAt(value: 110).change?.tone.color == Colors.green)
        #expect(GemChartData.mock(base: 100, currency: .usd).headerAt(value: 90).change?.tone.color == Colors.red)
    }

    @Test
    func theSecondaryValueIsTheOneTheChangeIsMeasuredAgainst() {
        #expect(GemChartData.mock(currency: .usd).headerAt(value: 100).secondaryValue == nil)
        #expect(GemChartData.mock(valueType: .priceChange, base: 100, showsSecondaryValue: true, currency: .usd).headerAt(value: 1500).secondaryValue?.text() == "$1,500.00")
    }
}
