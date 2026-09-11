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
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100), valueType: .priceChange).priceText == "+$100.00")
        #expect(ChartHeaderViewModel.mock(header: .mock(value: -50), valueType: .priceChange).priceText == "-$50.00")
    }

    @Test
    func priceColor() {
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100)).priceColor == Colors.black)
        #expect(ChartHeaderViewModel.mock(header: .mock(value: 100), valueType: .priceChange).priceColor == Colors.green)
    }

    @Test
    func priceChangeText() {
        #expect(ChartHeaderViewModel.mock(header: .mock(changePercentage: 5.5)).priceChangeText == "+5.50%")
        #expect(ChartHeaderViewModel.mock(header: .mock(changePercentage: nil)).priceChangeText == nil)
        #expect(ChartHeaderViewModel.mock(header: .mock(changePercentage: 10), valueType: .priceChange).priceChangeText == "(10.00%)")
    }

    @Test
    func priceChangeTextColor() {
        #expect(ChartHeaderViewModel.mock(header: .mock(changePercentage: 10)).priceChangeTextColor == Colors.green)
        #expect(ChartHeaderViewModel.mock(header: .mock(changePercentage: -10)).priceChangeTextColor == Colors.red)
    }

    @Test
    func dateText() {
        #expect(ChartHeaderViewModel.mock(date: nil).dateText == nil)
        #expect(ChartHeaderViewModel.mock(date: Date()).dateText != nil)
    }

    @Test
    func headerValueText() {
        #expect(ChartHeaderViewModel.mock().headerValueText == nil)
        #expect(ChartHeaderViewModel.mock(header: .mock(secondaryValue: 1500)).headerValueText == "$1,500.00")
    }
}
