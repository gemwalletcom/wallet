// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import Testing

struct CandlestickChartViewModelTests {
    @Test
    func labelOffsetReturnsToTheEdgeAfterAGap() {
        let model = CandlestickChartViewModel(
            candles: [.mock(high: 200, low: 100)],
            lines: [
                ChartLineViewModel(line: ChartLine(type: .liquidation, price: 120), formatter: NumericFormatter()),
                ChartLineViewModel(line: ChartLine(type: .entry, price: 121), formatter: NumericFormatter()),
                ChartLineViewModel(line: ChartLine(type: .takeProfit, price: 180), formatter: NumericFormatter()),
            ],
            formatter: CurrencyFormatter(currencyCode: "USD"),
        )

        #expect(model.lineLabelOffsets.first == 0)
        #expect(model.lineLabelOffsets.dropFirst().first == CandlestickChartViewModel.Constants.labelOverlapSpacing)
        #expect(model.lineLabelOffsets.last == 0)
    }

    @Test
    func flatSeriesKeepsAMeasurableRange() {
        let model = CandlestickChartViewModel(
            candles: [.mock(open: 100, high: 100, low: 100, close: 100)],
            formatter: CurrencyFormatter(currencyCode: "USD"),
        )

        #expect(model.yAxisRange.lowerBound < model.yAxisRange.upperBound)
    }
}
