// Copyright (c). Gem Wallet. All rights reserved.

import Charts
import Components
import struct Gemstone.ChartCandleStick
import struct Gemstone.GemCandleChart
import struct Gemstone.GemCandleTooltip
import PrimitivesComponents
import Style
import SwiftUI

private enum ChartKey {
    static let date = "Date"
    static let low = "Low"
    static let high = "High"
    static let open = "Open"
    static let close = "Close"
    static let price = "Price"
}

struct CandlestickChartView: View {
    private enum Metrics {
        static let labelOverlapSpacing: CGFloat = 115
        static let lineStyle = StrokeStyle(lineWidth: 1, dash: [4, 3])
        static let dateLabelWidth: CGFloat = 60
        static let candleBodyWidthRatio: Double = 0.6
    }

    private let chart: GemCandleChart
    private let timeAxis: ChartTimeAxis
    private let dateFormatter = ChartDateFormatter()
    private let onZoom: @MainActor (Double) -> Void

    @Binding private var isPinching: Bool
    @State private var selectedIndex: Int?

    init(chart: GemCandleChart, isPinching: Binding<Bool>, onZoom: @escaping @MainActor (Double) -> Void) {
        self.chart = chart
        timeAxis = ChartTimeAxis(ticks: chart.xTicks, format: chart.xTickFormat)
        _isPinching = isPinching
        self.onZoom = onZoom
    }

    private var selectedCandle: ChartCandleStick? {
        selectedIndex.flatMap { chart.candles[safe: $0] }
    }

    var body: some View {
        VStack {
            priceHeader
            chartView
                .padding(.bottom, Spacing.small)
        }
        .sensoryFeedback(.selection, trigger: selectedCandle?.date) { _, date in date != nil }
    }

    private var priceHeader: some View {
        VStack {
            if let selection = selectedIndex.flatMap({ chart.selection(index: UInt32($0)) }) {
                ChartHeaderView(header: selection.header, date: dateFormatter.string(for: selection.date, style: chart.dateStyle))
            } else {
                ChartHeaderView(header: chart.header)
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private var chartView: some View {
        Chart {
            currentPriceMark
            candlestickMarks
            linesMarks
            selectionMarks
        }
        .chartOverlay { proxy in
            GeometryReader { geometry in
                Color.clear
                    .chartGestures(
                        isPinching: $isPinching,
                        onScrub: { location in
                            if let index = findCandle(location: location, proxy: proxy, geometry: geometry) {
                                selectedIndex = index
                            }
                        },
                        onScrubEnd: { selectedIndex = nil },
                        onZoom: onZoom,
                    )

                if let selectedCandle, let tooltip = selectedIndex.flatMap({ chart.tooltip(index: UInt32($0)) }) {
                    tooltipOverlay(tooltip, for: selectedCandle, proxy: proxy, geometry: geometry)
                }
            }
        }
        .chartXAxis {
            AxisMarks(position: .bottom, values: timeAxis.ticks) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel(horizontalSpacing: -Metrics.dateLabelWidth / 2, verticalSpacing: Spacing.small) {
                    if let date = value.as(Date.self) {
                        Text(timeAxis.label(for: date))
                            .font(.caption2)
                            .foregroundStyle(Colors.gray)
                            .fixedSize()
                            .frame(width: Metrics.dateLabelWidth)
                    }
                }
            }
        }
        .chartYAxis {
            AxisMarks(position: .trailing, values: chart.layout.ticks.map(\.value)) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisTick(stroke: StrokeStyle(lineWidth: ChartGridStyle.lineWidth))
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel {
                    Text(chart.layout.ticks[safe: value.index]?.text() ?? "")
                        .font(.caption2)
                        .foregroundStyle(Colors.gray)
                        .padding(.horizontal, .extraSmall)
                }
            }
            if let currentPrice = chart.layout.currentPrice {
                AxisMarks(position: .trailing, values: [currentPrice.value]) { _ in
                    AxisValueLabel {
                        Text(currentPrice.text())
                            .font(.caption2)
                            .foregroundStyle(Colors.whiteSolid)
                            .padding(.horizontal, .extraSmall)
                            .padding(.vertical, .space1)
                            .background(chart.layout.tones.last?.color ?? Colors.gray)
                            .clipShape(RoundedRectangle(cornerRadius: Spacing.tiny))
                    }
                }
            }
        }
        .chartXScale(domain: xAxisRange)
        .chartYScale(domain: chart.layout.priceLow ... chart.layout.priceHigh)
    }

    @ChartContentBuilder
    private var currentPriceMark: some ChartContent {
        if let currentPrice = chart.layout.currentPrice {
            RuleMark(y: .value(ChartKey.price, currentPrice.value))
                .foregroundStyle(Colors.gray.opacity(.semiStrong))
                .lineStyle(StrokeStyle(lineWidth: 1, dash: [2, 3]))
        }
    }

    @ChartContentBuilder
    private var candlestickMarks: some ChartContent {
        ForEach(Array(zip(chart.candles, chart.layout.tones)), id: \.0.date) { candle, tone in
            let color = tone.color
            RuleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.low, candle.low),
                yEnd: .value(ChartKey.high, candle.high),
            )
            .lineStyle(StrokeStyle(lineWidth: .space1))
            .foregroundStyle(color)

            RectangleMark(
                xStart: .value(ChartKey.date, candle.date.addingTimeInterval(-bodyHalfWidth)),
                xEnd: .value(ChartKey.date, candle.date.addingTimeInterval(bodyHalfWidth)),
                yStart: .value(ChartKey.open, candle.open),
                yEnd: .value(ChartKey.close, candle.close),
            )
            .foregroundStyle(color)
        }
    }

    @ChartContentBuilder
    private var linesMarks: some ChartContent {
        ForEach(chart.layout.lines, id: \.kind) { line in
            RuleMark(y: .value(ChartKey.price, line.price.value))
                .foregroundStyle(line.kind.color.opacity(.semiStrong))
                .lineStyle(Metrics.lineStyle)
        }

        ForEach(chart.layout.lines, id: \.kind) { line in
            RuleMark(y: .value(ChartKey.price, line.price.value))
                .foregroundStyle(.clear)
                .annotation(position: .overlay, alignment: .leading, spacing: .zero) {
                    Text(line.label.text)
                        .font(.app.caption)
                        .foregroundStyle(Colors.whiteSolid)
                        .padding(.tiny)
                        .background(line.kind.color)
                        .clipShape(RoundedRectangle(cornerRadius: .tiny))
                        .offset(x: CGFloat(line.overlapLevel) * Metrics.labelOverlapSpacing)
                }
        }
    }

    @ChartContentBuilder
    private var selectionMarks: some ChartContent {
        if let selectedCandle {
            PointMark(
                x: .value(ChartKey.date, selectedCandle.date),
                y: .value(ChartKey.price, selectedCandle.close),
            )
            .symbol {
                Circle()
                    .strokeBorder(Colors.blue, lineWidth: .space2)
                    .background(Circle().foregroundStyle(Colors.white))
                    .frame(width: .space12)
            }

            RuleMark(x: .value(ChartKey.date, selectedCandle.date))
                .foregroundStyle(Colors.blue)
                .lineStyle(StrokeStyle(lineWidth: .space1, dash: [5]))
        }
    }

    @ViewBuilder
    private func tooltipOverlay(_ tooltip: GemCandleTooltip, for candle: ChartCandleStick, proxy: ChartProxy, geometry: GeometryProxy) -> some View {
        let isRightHalf: Bool = {
            guard let plotFrame = proxy.plotFrame,
                  let xPosition = proxy.position(forX: candle.date) else { return false }
            return xPosition > geometry[plotFrame].size.width / 2
        }()

        CandleTooltipView(tooltip: tooltip)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: isRightHalf ? .topLeading : .topTrailing)
            .padding(.leading, Spacing.small)
            .padding(.top, Spacing.small)
            .padding(.trailing, Spacing.extraLarge + Spacing.medium)
            .transition(.opacity)
            .animation(.easeInOut(duration: Interval.AnimationDuration.fast), value: isRightHalf)
            .allowsHitTesting(false)
    }

    private var xAxisRange: ClosedRange<Date> {
        chart.start ... chart.end
    }

    private var bodyHalfWidth: TimeInterval {
        TimeInterval(chart.intervalSeconds) * Metrics.candleBodyWidthRatio / 2
    }

    private func findCandle(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) -> Int? {
        guard let plotFrame = proxy.plotFrame else { return nil }
        let relativeX = location.x - geometry[plotFrame].origin.x
        guard let date = proxy.value(atX: relativeX) as Date? else { return nil }
        return chart.candles.indices.min { abs(chart.candles[$0].date.timeIntervalSince(date)) < abs(chart.candles[$1].date.timeIntervalSince(date)) }
    }
}
