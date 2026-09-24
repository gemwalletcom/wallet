// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.candlestickHeader
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
    }

    let candles: [ChartCandleStick]

    private let layout: GemPerpetualChartLayout
    private let period: ChartPeriod
    private let dateFormatter = ChartDateFormatter()

    init(
        candles: [ChartCandleStick],
        period: ChartPeriod = .day,
        position: PerpetualPosition? = nil,
    ) {
        self.candles = candles
        layout = perpetualChartLayout(candles: candles.map { $0.toGem() }, position: position?.toGem())
        self.period = period
    }

    var xAxisRange: ClosedRange<Date> {
        (candles.first?.date ?? Date()) ... (candles.last?.date ?? Date())
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

    var xAxisTickCount: Int {
        Int(layout.xTickCount)
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
        guard let target = selectedCandle ?? candles.last, let base = candles.first?.close else { return nil }
        return candlestickHeader(base: base, value: target.close)
    }

    func dateText(for selectedCandle: ChartCandleStick?) -> String? {
        selectedCandle.map { dateFormatter.string(for: $0.date, period: period) }
    }

    func tooltipModel(for candle: ChartCandleStick) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle)
    }

    func candle(for date: Date) -> ChartCandleStick? {
        candles.min { abs($0.date.timeIntervalSince(date)) < abs($1.date.timeIntervalSince(date)) }
    }
}
