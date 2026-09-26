// Copyright (c). Gem Wallet. All rights reserved.

import Charts
import Components
import struct Gemstone.ChartCandleStick
import struct Gemstone.GemCandleChart
import GemstonePrimitives
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
    }

    private let chart: GemCandleChart
    private let dateFormatter = ChartDateFormatter()

    @State private var selectedIndex: Int?

    init(chart: GemCandleChart) {
        self.chart = chart
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
            candlestickMarks
            linesMarks
            selectionMarks
        }
        .chartOverlay { proxy in
            GeometryReader { geometry in
                Rectangle()
                    .fill(.clear)
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { value in
                                if let index = findCandle(location: value.location, proxy: proxy, geometry: geometry) {
                                    selectedIndex = index
                                }
                            }
                            .onEnded { _ in
                                selectedIndex = nil
                            },
                    )

                if let selectedCandle {
                    tooltipOverlay(for: selectedCandle, proxy: proxy, geometry: geometry)
                }
            }
        }
        .chartXAxis {
            AxisMarks(position: .bottom, values: .automatic(desiredCount: Int(chart.layout.xTickCount))) { _ in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
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
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.open, candle.open),
                yEnd: .value(ChartKey.close, candle.close),
                width: .fixed(.space4),
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
    private func tooltipOverlay(for candle: ChartCandleStick, proxy: ChartProxy, geometry: GeometryProxy) -> some View {
        let isRightHalf: Bool = {
            guard let plotFrame = proxy.plotFrame,
                  let xPosition = proxy.position(forX: candle.date) else { return false }
            return xPosition > geometry[plotFrame].size.width / 2
        }()

        CandleTooltipView(model: CandleTooltipViewModel(candle: candle.toPrimitives()))
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: isRightHalf ? .topLeading : .topTrailing)
            .padding(.leading, Spacing.small)
            .padding(.top, Spacing.small)
            .padding(.trailing, Spacing.extraLarge + Spacing.medium)
            .transition(.opacity)
            .animation(.easeInOut(duration: Interval.AnimationDuration.fast), value: isRightHalf)
            .allowsHitTesting(false)
    }

    private var xAxisRange: ClosedRange<Date> {
        (chart.candles.first?.date ?? Date()) ... (chart.candles.last?.date ?? Date())
    }

    private func findCandle(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) -> Int? {
        guard let plotFrame = proxy.plotFrame else { return nil }
        let relativeX = location.x - geometry[plotFrame].origin.x
        guard let date = proxy.value(atX: relativeX) as Date? else { return nil }
        return chart.candles.indices.min { abs(chart.candles[$0].date.timeIntervalSince(date)) < abs(chart.candles[$1].date.timeIntervalSince(date)) }
    }
}
