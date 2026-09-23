// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.candlestickHeader
import struct Gemstone.GemCandleViewport
import struct Gemstone.GemChartHeader
import struct Gemstone.GemPerpetualChartLayout
import func Gemstone.perpetualChartLayout
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickChartViewModel {
    enum Constants {
        static let labelOverlapSpacing: CGFloat = 115
        static let candleBodyWidthRatio: Double = 0.6
    }

    let candles: [ChartCandleStick]

    private let viewport: GemCandleViewport
    private let base: Double
    private let layout: GemPerpetualChartLayout
    private let timeAxis: ChartTimeAxis
    private let period: ChartPeriod
    private let dateFormatter = ChartDateFormatter()

    init(
        viewport: GemCandleViewport,
        base: Double,
        period: ChartPeriod = .day,
        position: PerpetualPosition? = nil,
    ) {
        self.viewport = viewport
        self.base = base
        candles = viewport.candles.map { $0.toPrimitives() }
        layout = perpetualChartLayout(candles: viewport.candles, position: position?.toGem())
        timeAxis = ChartTimeAxis(dates: viewport.candles.map(\.date), range: viewport.start ... viewport.end, interval: TimeInterval(viewport.intervalSeconds))
        self.period = period
    }

    var xAxisRange: ClosedRange<Date> {
        viewport.start ... viewport.end
    }

    var yAxisRange: ClosedRange<Double> {
        layout.priceLow ... layout.priceHigh
    }

    var lines: [ChartLineViewModel] {
        layout.lines.map { ChartLineViewModel(line: $0) }
    }

    var yAxisTicks: [Double] {
        layout.ticks.map(\.value)
    }

    func yAxisTickText(at index: Int) -> String {
        layout.ticks[safe: index]?.text() ?? ""
    }

    var xAxisTicks: [Date] {
        timeAxis.ticks
    }

    func xAxisLabel(for date: Date) -> String {
        timeAxis.label(for: date)
    }

    var lineLabelOffsets: [CGFloat] {
        layout.lines.map { CGFloat($0.overlapLevel) * Constants.labelOverlapSpacing }
    }

    var currentPrice: Double? {
        layout.currentPrice?.value
    }

    var currentPriceText: String {
        layout.currentPrice?.text() ?? ""
    }

    var currentPriceColor: Color {
        layout.tones.last?.color ?? Colors.gray
    }

    var candleMarks: [(candle: ChartCandleStick, color: Color)] {
        zip(candles, layout.tones).map { ($0, $1.color) }
    }

    func header(for selectedCandle: ChartCandleStick?) -> GemChartHeader? {
        guard let target = selectedCandle ?? candles.last else { return nil }
        return candlestickHeader(base: base, value: target.close)
    }

    func dateText(for selectedCandle: ChartCandleStick?) -> String? {
        selectedCandle.map { dateFormatter.string(for: $0.date, period: period) }
    }

    func tooltipModel(for candle: ChartCandleStick) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle)
    }

    func bodyStart(for candle: ChartCandleStick) -> Date {
        candle.date.addingTimeInterval(-bodyHalfWidth)
    }

    func bodyEnd(for candle: ChartCandleStick) -> Date {
        candle.date.addingTimeInterval(bodyHalfWidth)
    }

    private var bodyHalfWidth: TimeInterval {
        TimeInterval(viewport.intervalSeconds) * Constants.candleBodyWidthRatio / 2
    }

    func candle(for date: Date) -> ChartCandleStick? {
        candles.min { abs($0.date.timeIntervalSince(date)) < abs($1.date.timeIntervalSince(date)) }
    }
}
