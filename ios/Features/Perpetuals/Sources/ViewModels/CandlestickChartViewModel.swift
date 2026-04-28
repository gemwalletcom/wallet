// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickChartViewModel {
    private static let labelOverlapPriceFraction = 0.06
    private static let labelOverlapSpacing: CGFloat = 115

    let candles: [ChartCandleStick]

    private let lines: [ChartLineViewModel]
    private let period: ChartPeriod
    private let formatter: CurrencyFormatter

    init(
        candles: [ChartCandleStick],
        period: ChartPeriod = .day,
        lines: [ChartLineViewModel] = [],
        formatter: CurrencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue),
    ) {
        self.candles = candles
        self.lines = lines
        self.period = period
        self.formatter = formatter
    }

    private var bounds: ChartBounds {
        ChartBounds(candles: candles, lines: lines)
    }

    var xAxisRange: ClosedRange<Date> {
        (candles.first?.date ?? Date()) ... (candles.last?.date ?? Date())
    }

    var yAxisRange: ClosedRange<Double> {
        bounds.minPrice ... bounds.maxPrice
    }

    var visibleLines: [ChartLineViewModel] {
        bounds.visibleLines
    }

    var priceAxisFormat: FloatingPointFormatStyle<Double> {
        bounds.axisFormat
    }

    var lineLabelOffsets: [CGFloat] {
        let bounds = bounds
        let visible = bounds.visibleLines
        let threshold = (bounds.maxPrice - bounds.minPrice) * Self.labelOverlapPriceFraction
        return visible.indices.reduce(into: [CGFloat]()) { offsets, index in
            let previous = offsets.last ?? 0
            let overlapsPrevious = index > 0 && abs(visible[index].price - visible[index - 1].price) < threshold
            offsets.append(overlapsPrevious ? previous + Self.labelOverlapSpacing : previous)
        }
    }

    var currentPrice: Double? { candles.last?.close }
    var currentPriceColor: Color { candles.last.map(candleColor(for:)) ?? Colors.gray }

    func headerModel(selectedCandle: ChartCandleStick?) -> ChartHeaderViewModel? {
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
