// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct ChartValuesViewModelTests {
    @Test
    func boundsComeFromTheChartData() {
        let model = ChartValuesViewModel.mock(chartData: .mock(values: [100, 200]))

        #expect(model.lowerBoundValueText == "$100.00")
        #expect(model.upperBoundValueText == "$200.00")
    }

    @Test
    func headerComesFromCore() {
        let model = ChartValuesViewModel.mock(chartData: .mock(values: [100, 200], header: .mock(value: 150)))

        #expect(model.chartHeaderViewModel?.header.value == 150)
        #expect(model.headerViewModel(for: model.charts[1]).header.value == 200)
        #expect(model.headerViewModel(for: model.charts[1]).header.changePercentage == 100)

        #expect(ChartValuesViewModel.mock(chartData: .mock(header: nil)).chartHeaderViewModel == nil)
    }
}
