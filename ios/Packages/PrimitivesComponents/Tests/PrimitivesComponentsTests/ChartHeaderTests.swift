// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartHeader
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct ChartHeaderTests {
    @Test
    func theValueReadsAsAPriceOrAsAChange() {
        #expect(GemChartHeader.mock(value: 100).value.text() == "$100.00")
        #expect(GemChartHeader.mock(value: 100, base: 0, valueType: .priceChange).value.text() == "+$100.00")
        #expect(GemChartHeader.mock(value: -50, base: 0, valueType: .priceChange).value.text() == "-$50.00")
    }

    @Test
    func onlyAChangeIsToned() {
        #expect(GemChartHeader.mock(value: 100).value.tone.color == Colors.black)
        #expect(GemChartHeader.mock(value: 100, base: 0, valueType: .priceChange).value.tone.color == Colors.green)
    }

    @Test
    func theChangeCarriesItsSignBracketsAndTone() {
        #expect(GemChartHeader.mock(value: 105.5, base: 100).change?.text() == "+5.50%")
        #expect(GemChartHeader.mock(value: 0, base: 0).change == nil)
        #expect(GemChartHeader.mock(value: 110, base: 100, valueType: .priceChange, showsSecondaryValue: true).change?.text() == "(10.00%)")
        #expect(GemChartHeader.mock(value: 110, base: 100).change?.tone.color == Colors.green)
        #expect(GemChartHeader.mock(value: 90, base: 100).change?.tone.color == Colors.red)
    }

    @Test
    func theSecondaryValueIsTheOneTheChangeIsMeasuredAgainst() {
        #expect(GemChartHeader.mock().secondaryValue == nil)
        #expect(GemChartHeader.mock(value: 1500, base: 100, valueType: .priceChange, showsSecondaryValue: true).secondaryValue?.text() == "$1,500.00")
    }
}
