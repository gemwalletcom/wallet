// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemChartHeader
import struct Gemstone.GemPerpetualChartLayout
import func Gemstone.perpetualChartLayout
import class Gemstone.PriceChangeCalculator
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickChartViewModel {
    private let priceChangeCalculator = PriceChangeCalculator()
    enum Constants {
        static let labelOverlapSpacing: CGFloat = 115
        static let xAxisTickCount = 6
    }

    let candles: [ChartCandleStick]

    private let layout: GemPerpetualChartLayout
    private let period: ChartPeriod
    private let formatter: CurrencyFormatter
    private let numericFormatter: NumericFormatter

    init(
        candles: [ChartCandleStick],
        period: ChartPeriod = .day,
        position: PerpetualPosition? = nil,
        formatter: CurrencyFormatter,
        numericFormatter: NumericFormatter = NumericFormatter(),
    ) {
        self.candles = candles
        layout = perpetualChartLayout(candles: candles.map { $0.map() }, position: position?.map())
        self.period = period
        self.formatter = formatter
        self.numericFormatter = numericFormatter
    }

    var xAxisRange: ClosedRange<Date> {
        (candles.first?.date ?? Date()) ... (candles.last?.date ?? Date())
    }

    var yAxisRange: ClosedRange<Double> {
        layout.priceLow ... layout.priceHigh
    }

    var lines: [ChartLineViewModel] {
        layout.lines.map { ChartLineViewModel(line: $0, formatter: numericFormatter) }
    }

    var yAxisTicks: [Double] {
        layout.ticks
    }

    func formattedPrice(_ price: Double) -> String {
        numericFormatter.string(price)
    }

    var lineLabelOffsets: [CGFloat] {
        layout.lines.map { CGFloat($0.overlapLevel) * Constants.labelOverlapSpacing }
    }

    var currentPrice: Double? {
        candles.last?.close
    }

    var currentPriceColor: Color {
        candles.last.map(candleColor(for:)) ?? Colors.gray
    }

    func headerModel(for selectedCandle: ChartCandleStick?) -> ChartHeaderViewModel? {
        guard let target = selectedCandle ?? candles.last, let base = candles.first?.close else { return nil }
        let changePercentage = priceChangeCalculator.percentage(from: base, to: target.close)
        return ChartHeaderViewModel(
            period: period,
            date: selectedCandle?.date,
            header: GemChartHeader(value: target.close, secondaryValue: nil, changePercentage: target.close == 0 ? nil : changePercentage),
            formatter: formatter,
        )
    }

    func tooltipModel(for candle: ChartCandleStick) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle, formatter: numericFormatter)
    }

    func candleColor(for candle: ChartCandleStick) -> Color {
        PriceChangeColor.color(for: candle.close - candle.open)
    }

    func candle(for date: Date) -> ChartCandleStick? {
        candles.min { abs($0.date.timeIntervalSince(date)) < abs($1.date.timeIntervalSince(date)) }
    }
}
