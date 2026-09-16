// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct ChartHeaderViewModelTests {
    @Test
    func priceText() {
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100)).priceText == "$100.00")
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100, base: 0, valueType: .priceChange), valueType: .priceChange).priceText == "+$100.00")
        #expect(ChartHeaderViewModel.mock(header: .mock(value: -50, base: 0, valueType: .priceChange), valueType: .priceChange).priceText == "-$50.00")
    }

    @Test
    func priceColor() {
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100)).priceColor == Colors.black)
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100, base: 0, valueType: .priceChange), valueType: .priceChange).priceColor == Colors.green)
    }

    @Test
    func priceChangeText() {
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 105.5, base: 100)).priceChangeText == "+5.50%")
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 0, base: 0)).priceChangeText == nil)
        #expect(
            ChartHeaderViewModel.mock(
                header: .mock(value: 110, base: 100, valueType: .priceChange, showsSecondaryValue: true),
                valueType: .priceChange,
            ).priceChangeText == "(10.00%)",
        )
    }

    @Test
    func priceChangeTextColor() {
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 110, base: 100)).priceChangeTextColor == Colors.green)
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 90, base: 100)).priceChangeTextColor == Colors.red)
    }

    @Test
    func dateText() {
        #expect(ChartHeaderViewModel.mock(date: nil).dateText == nil)
        #expect(ChartHeaderViewModel.mock(date: Date()).dateText != nil)
    }

    @Test
    func headerValueText() {
        #expect(ChartHeaderViewModel.mock().headerValueText == nil)
        #expect(
            ChartHeaderViewModel.mock(
                header: .mock(value: 1500, base: 100, valueType: .priceChange, showsSecondaryValue: true),
            ).headerValueText == "$1,500.00",
        )
    }
}
