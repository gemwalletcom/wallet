// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickChartViewModel {
    enum Constants {
        static let labelOverlapPriceFraction = 0.06
        static let labelOverlapSpacing: CGFloat = 115
        static let xAxisTickCount = 6
        static let yAxisTickCount = 4
    }

    let candles: [ChartCandleStick]

    private let lines: [ChartLineViewModel]
    private let period: ChartPeriod
    private let formatter: CurrencyFormatter

    init(
        candles: [ChartCandleStick],
        period: ChartPeriod = .day,
        lines: [ChartLineViewModel] = [],
        formatter: CurrencyFormatter,
    ) {
        self.candles = candles
        self.lines = lines
        self.period = period
        self.formatter = formatter
    }

    var xAxisRange: ClosedRange<Date> {
        (candles.first?.date ?? Date()) ... (candles.last?.date ?? Date())
    }

    var yAxisRange: ClosedRange<Double> {
        let linePrices = visibleLines.map(\.price)
        let lowest = min(candleMin, linePrices.min() ?? candleMin)
        let highest = max(candleMax, linePrices.max() ?? candleMax)
        let padding = (highest - lowest) * 0.05
        let lowerBound = max(lowest - padding, lowest * 0.95)
        return lowerBound ... (highest + padding)
    }

    var visibleLines: [ChartLineViewModel] {
        let buffer = (candleMax - candleMin) * 0.5
        return lines
            .filter { $0.price >= candleMin - buffer && $0.price <= candleMax + buffer }
            .sorted { $0.price < $1.price }
    }

    var yAxisTicks: [Double] {
        guard candleMax > candleMin else { return [candleMin, candleMax] }
        let step = (candleMax - candleMin) / Double(Constants.yAxisTickCount - 1)
        return (0 ..< Constants.yAxisTickCount).map { candleMin + Double($0) * step }
    }

    private var candleMin: Double {
        candles.map(\.low).min() ?? 0
    }

    private var candleMax: Double {
        candles.map(\.high).max() ?? 1
    }

    func formattedPrice(_ price: Double) -> String {
        formatter.string(double: price, symbol: nil)
    }

    var lineLabelOffsets: [CGFloat] {
        let visible = visibleLines
        let range = yAxisRange
        let threshold = (range.upperBound - range.lowerBound) * Constants.labelOverlapPriceFraction
        return visible.indices.reduce(into: [CGFloat]()) { offsets, index in
            let previous = offsets.last ?? 0
            let overlapsPrevious = index > 0 && abs(visible[index].price - visible[index - 1].price) < threshold
            offsets.append(overlapsPrevious ? previous + Constants.labelOverlapSpacing : previous)
        }
    }

    var currentPrice: Double? {
        candles.last?.close
    }

    var currentPriceColor: Color {
        candles.last.map(candleColor(for:)) ?? Colors.gray
    }

    func headerModel(for selectedCandle: ChartCandleStick?) -> ChartHeaderViewModel? {
        guard let target = selectedCandle ?? candles.last, let base = candles.first?.close else { return nil }
        return ChartHeaderViewModel(
            period: period,
            date: selectedCandle?.date,
            price: target.close,
            priceChangePercentage: PriceChangeCalculator.calculate(.percentage(from: base, to: target.close)),
            formatter: formatter,
        )
    }

    func tooltipModel(for candle: ChartCandleStick) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle, formatter: formatter)
    }

    func candleColor(for candle: ChartCandleStick) -> Color {
        PriceChangeColor.color(for: candle.close - candle.open)
    }

    func candle(for date: Date) -> ChartCandleStick? {
        candles.min { abs($0.date.timeIntervalSince(date)) < abs($1.date.timeIntervalSince(date)) }
    }
}
