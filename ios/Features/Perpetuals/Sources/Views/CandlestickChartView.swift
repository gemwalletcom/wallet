// Copyright (c). Gem Wallet. All rights reserved.

import Charts
import Components
import Primitives
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
        static let dateLabelWidth: CGFloat = 60
    }

    private let model: CandlestickChartViewModel
    private let onZoom: @MainActor (Double) -> Void

    @Binding private var isPinching: Bool
    @State private var selectedCandle: ChartCandleStick?

    init(model: CandlestickChartViewModel, isPinching: Binding<Bool>, onZoom: @escaping @MainActor (Double) -> Void) {
        self.model = model
        _isPinching = isPinching
        self.onZoom = onZoom
    }

    var body: some View {
        VStack {
            priceHeader
            chart
                .padding(.bottom, Spacing.small)
        }
        .sensoryFeedback(.selection, trigger: selectedCandle?.date) { _, date in date != nil }
    }

    private var priceHeader: some View {
        VStack {
            if let header = model.header(for: selectedCandle) {
                ChartHeaderView(header: header, date: model.dateText(for: selectedCandle))
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private var chart: some View {
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
                            if let candle = findCandle(location: location, proxy: proxy, geometry: geometry) {
                                selectedCandle = candle
                            }
                        },
                        onScrubEnd: { selectedCandle = nil },
                        onZoom: onZoom,
                    )

                if let selectedCandle {
                    tooltipOverlay(for: selectedCandle, proxy: proxy, geometry: geometry)
                }
            }
        }
        .chartXAxis {
            AxisMarks(position: .bottom, values: model.xAxisTicks) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel(horizontalSpacing: -Metrics.dateLabelWidth / 2, verticalSpacing: Spacing.small) {
                    if let date = value.as(Date.self) {
                        Text(model.xAxisLabel(for: date))
                            .font(.caption2)
                            .foregroundStyle(Colors.gray)
                            .fixedSize()
                            .frame(width: Metrics.dateLabelWidth)
                    }
                }
            }
        }
        .chartYAxis {
            AxisMarks(position: .trailing, values: model.yAxisTicks) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisTick(stroke: StrokeStyle(lineWidth: ChartGridStyle.lineWidth))
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel {
                    Text(model.yAxisTickText(at: value.index))
                        .font(.caption2)
                        .foregroundStyle(Colors.gray)
                        .padding(.horizontal, .extraSmall)
                }
            }
            if let currentPrice = model.currentPrice {
                AxisMarks(position: .trailing, values: [currentPrice]) { _ in
                    AxisValueLabel {
                        Text(model.currentPriceText)
                            .font(.caption2)
                            .foregroundStyle(Colors.whiteSolid)
                            .padding(.horizontal, .extraSmall)
                            .padding(.vertical, .space1)
                            .background(model.currentPriceColor)
                            .clipShape(RoundedRectangle(cornerRadius: Spacing.tiny))
                    }
                }
            }
        }
        .chartXScale(domain: model.xAxisRange)
        .chartYScale(domain: model.yAxisRange)
    }

    @ChartContentBuilder
    private var currentPriceMark: some ChartContent {
        if let currentPrice = model.currentPrice {
            RuleMark(y: .value(ChartKey.price, currentPrice))
                .foregroundStyle(Colors.gray.opacity(.semiStrong))
                .lineStyle(StrokeStyle(lineWidth: 1, dash: [2, 3]))
        }
    }

    @ChartContentBuilder
    private var candlestickMarks: some ChartContent {
        ForEach(model.candleMarks, id: \.candle.date) { candle, color in
            RuleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.low, candle.low),
                yEnd: .value(ChartKey.high, candle.high),
            )
            .lineStyle(StrokeStyle(lineWidth: .space1))
            .foregroundStyle(color)

            RectangleMark(
                xStart: .value(ChartKey.date, model.bodyStart(for: candle)),
                xEnd: .value(ChartKey.date, model.bodyEnd(for: candle)),
                yStart: .value(ChartKey.open, candle.open),
                yEnd: .value(ChartKey.close, candle.close),
            )
            .foregroundStyle(color)
        }
    }

    @ChartContentBuilder
    private var linesMarks: some ChartContent {
        ForEach(model.lines) { line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(line.color.opacity(.semiStrong))
                .lineStyle(line.lineStyle)
        }

        ForEach(Array(model.lines.enumerated()), id: \.element.id) { index, line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(.clear)
                .annotation(position: .overlay, alignment: .leading, spacing: .zero) {
                    Text(line.label)
                        .font(.app.caption)
                        .foregroundStyle(Colors.whiteSolid)
                        .padding(.tiny)
                        .background(line.color)
                        .clipShape(RoundedRectangle(cornerRadius: .tiny))
                        .offset(x: model.lineLabelOffsets[index])
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

        CandleTooltipView(model: model.tooltipModel(for: candle))
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: isRightHalf ? .topLeading : .topTrailing)
            .padding(.leading, Spacing.small)
            .padding(.top, Spacing.small)
            .padding(.trailing, Spacing.extraLarge + Spacing.medium)
            .transition(.opacity)
            .animation(.easeInOut(duration: Interval.AnimationDuration.fast), value: isRightHalf)
            .allowsHitTesting(false)
    }

    private func findCandle(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) -> ChartCandleStick? {
        guard let plotFrame = proxy.plotFrame else { return nil }
        let relativeX = location.x - geometry[plotFrame].origin.x
        guard let date = proxy.value(atX: relativeX) as Date? else { return nil }
        return model.candle(for: date)
    }
}
