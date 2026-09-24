// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartViewport
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct ChartValuesViewModelTests {
    @Test
    func boundsComeFromTheViewport() {
        let model = ChartValuesViewModel(period: .day, chartData: .mock(values: [100, 150, 80, 120]), viewport: .mock(chartData: .mock(values: [80, 120])))

        #expect(model.bounds.low.text() == "$80.00")
        #expect(model.bounds.high.text() == "$120.00", "a zoomed window labels its own high, not the period's")
        #expect(model.charts.map(\.value) == [80, 120])
    }

    @Test
    func scrubbingSelectsOnlyThePointsTheLineIsDrawnThrough() {
        let chartData = GemChartData.mock(values: [100, 150, 80, 120])
        let values = chartData.values
        let viewport = GemChartViewport(
            start: values[1].date,
            end: values[3].date,
            values: Array(values[1...]),
            renderValues: [values[0], values[1], values[3]],
            bounds: GemChartData.mock(values: [150, 80, 120]).bounds(),
        )
        let model = ChartValuesViewModel(period: .day, chartData: chartData, viewport: viewport)

        #expect(model.value(for: values[2].date.addingTimeInterval(600))?.value == 120, "a point the thinned line skips is never selected")
        #expect(model.value(for: values[1].date.addingTimeInterval(-3000))?.value == 150, "the point the line enters from, left of the window, is never selected")
    }

    @Test
    func theScaleAndTheLabelsFollowCoreBounds() {
        let model = ChartValuesViewModel.mock(chartData: .mock(values: [100, 150, 80, 120]))

        #expect(model.yScale == [76.5, 153.5])
        #expect(model.bounds.low.text() == "$80.00")
        #expect(model.bounds.high.text() == "$150.00")
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
