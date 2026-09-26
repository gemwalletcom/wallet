// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct ChartHeaderTests {
    private func header(valueType: GemChartValueType = .price, base: Double = 0, showsSecondaryValue: Bool = false, at value: Double) throws -> GemChartHeader {
        try #require(GemChartData.mock(
            valueType: valueType,
            base: base,
            showsSecondaryValue: showsSecondaryValue,
            currency: .usd,
            values: [Primitives.ChartDateValue.mock(value: value).toGem()],
        ).selection(index: 0)).header
    }

    @Test
    func theValueReadsAsAPriceOrAsAChange() throws {
        #expect(try header(at: 100).value.text() == "$100.00")
        #expect(try header(valueType: .priceChange, base: 0, at: 100).value.text() == "+$100.00")
        #expect(try header(valueType: .priceChange, base: 0, at: -50).value.text() == "-$50.00")
    }

    @Test
    func onlyAChangeIsToned() throws {
        #expect(try header(at: 100).value.tone.color == Colors.black)
        #expect(try header(valueType: .priceChange, base: 0, at: 100).value.tone.color == Colors.green)
    }

    @Test
    func theChangeCarriesItsSignBracketsAndTone() throws {
        #expect(try header(base: 100, at: 105.5).change?.text() == "+5.50%")
        #expect(try header(base: 0, at: 0).change == nil)
        #expect(try header(valueType: .priceChange, base: 100, showsSecondaryValue: true, at: 110).change?.text() == "(10.00%)")
        #expect(try header(base: 100, at: 110).change?.tone.color == Colors.green)
        #expect(try header(base: 100, at: 90).change?.tone.color == Colors.red)
    }

    @Test
    func theSecondaryValueIsTheOneTheChangeIsMeasuredAgainst() throws {
        #expect(try header(at: 100).secondaryValue == nil)
        #expect(try header(valueType: .priceChange, base: 100, showsSecondaryValue: true, at: 1500).secondaryValue?.text() == "$1,500.00")
    }
}
