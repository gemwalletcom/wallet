// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartCurrent
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct ChartValuesViewModelTests {
    @Test
    func chartValuesViewModel() {
        let model = ChartValuesViewModel.mock(
            current: GemChartCurrent(date: .now, value: 150, changePercentage: 4.2),
            values: .mock(values: [100, 200]),
        )

        #expect(model.lowerBoundValueText == "$100.00")
        #expect(model.upperBoundValueText == "$200.00")
        #expect(model.chartHeaderViewModel?.price == 150)
        #expect(model.chartHeaderViewModel?.priceChangePercentage == 4.2)
        #expect(model.chartHeaderViewModel?.dateText == nil)
        #expect(model.headerViewModel(for: ChartDateValue(date: Date(), value: 150)).priceChangePercentage == 50)
    }

    @Test
    func restingHeaderFallsBackToTheLastPoint() {
        let model = ChartValuesViewModel.mock(current: nil, values: .mock(values: [100, 200]))

        #expect(model.chartHeaderViewModel?.price == 200)
        #expect(model.chartHeaderViewModel?.priceChangePercentage == 100)
        #expect(model.chartHeaderViewModel?.dateText == nil)
    }

    @Test
    func headerValue() {
        let model = ChartValuesViewModel.mock(values: .mock(values: [100, 200]), headerValue: 500)

        #expect(model.chartHeaderViewModel?.headerValue == 500)
        #expect(model.headerViewModel(for: ChartDateValue(date: Date(), value: 150)).headerValue == 150)
        #expect(ChartValuesViewModel.mock().headerViewModel(for: ChartDateValue(date: Date(), value: 150)).headerValue == nil)
    }
}
