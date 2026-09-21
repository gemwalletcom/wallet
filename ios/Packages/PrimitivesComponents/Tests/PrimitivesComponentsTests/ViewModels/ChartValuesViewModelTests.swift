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
    func theScaleAndTheLabelsFollowCoreBounds() {
        let model = ChartValuesViewModel.mock(chartData: .mock(values: [100, 150, 80, 120]))

        #expect(model.yScale == [76.5, 153.5])
        #expect(model.lowerBoundValueText == "$80.00")
        #expect(model.upperBoundValueText == "$150.00")
        #expect(model.lowerBoundDate == model.charts[2].date)
        #expect(model.upperBoundDate == model.charts[1].date)
    }

    @Test
    func headerComesFromCore() {
        let model = ChartValuesViewModel.mock(chartData: .mock(values: [100, 200], header: .mock(value: 150)))

        #expect(model.chartHeader?.value.value == 150)
        #expect(model.header(for: model.charts[1]).value.value == 200)
        #expect(model.header(for: model.charts[1]).change?.value == 100)

        #expect(ChartValuesViewModel.mock(chartData: .mock(header: nil)).chartHeader == nil)
    }
}
