// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct ChartValuesViewModelTests {
    @Test
    func boundsComeFromTheChartData() {
        let model = ChartValuesViewModel.mock(chartData: .mock(
            base: 100,
            currency: .usd,
            values: [Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 0), value: 100).toGem(), Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 3600), value: 200).toGem()],
        ))

        #expect(model.bounds.low.text() == "$100.00")
        #expect(model.bounds.high.text() == "$200.00")
    }

    @Test
    func theScaleAndTheLabelsFollowCoreBounds() {
        let model = ChartValuesViewModel.mock(chartData: .mock(
            base: 100,
            currency: .usd,
            values: [
                Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 0), value: 100).toGem(),
                Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 3600), value: 150).toGem(),
                Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 7200), value: 80).toGem(),
                Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 10800), value: 120).toGem(),
            ],
        ))

        #expect(model.yScale == [76.5, 153.5])
        #expect(model.bounds.low.text() == "$80.00")
        #expect(model.bounds.high.text() == "$150.00")
        #expect(model.lowerBoundDate == model.charts[2].date)
        #expect(model.upperBoundDate == model.charts[1].date)
    }

    @Test
    func headerComesFromCore() {
        let model = ChartValuesViewModel.mock(chartData: .mock(
            base: 100,
            currency: .usd,
            values: [Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 0), value: 100).toGem(), Primitives.ChartDateValue.mock(date: Date(timeIntervalSince1970: 3600), value: 200).toGem()],
            header: .mock(value: .mock(value: 150)),
        ))

        #expect(model.chartHeader?.value.value == 150)
        #expect(model.header(for: model.charts[1]).value.value == 200)
        #expect(model.header(for: model.charts[1]).change?.value == 100)

        #expect(ChartValuesViewModel.mock(chartData: .mock(header: nil)).chartHeader == nil)
    }
}
