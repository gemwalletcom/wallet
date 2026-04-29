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
    private let model: CandlestickChartViewModel

    @State private var selectedCandle: ChartCandleStick? {
        didSet {
            if let selectedCandle, selectedCandle.date != oldValue?.date {
                vibrate()
            }
        }
    }

    init(model: CandlestickChartViewModel) {
        self.model = model
    }

    var body: some View {
        VStack {
            priceHeader
            chart
                .padding(.bottom, Spacing.small)
        }
    }

    private var priceHeader: some View {
        VStack {
            if let headerModel = model.headerModel(for: selectedCandle) {
                ChartHeaderView(model: headerModel)
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private var chart: some View {
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
                                if let candle = findCandle(location: value.location, proxy: proxy, geometry: geometry) {
                                    selectedCandle = candle
                                }
                            }
                            .onEnded { _ in
                                selectedCandle = nil
                            },
                    )

                if let selectedCandle {
                    tooltipOverlay(for: selectedCandle, proxy: proxy, geometry: geometry)
                }
            }
        }
        .chartXAxis {
            AxisMarks(position: .bottom, values: .automatic(desiredCount: CandlestickChartViewModel.Constants.xAxisTickCount)) { _ in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
            }
        }
        .chartYAxis {
            AxisMarks(position: .trailing, values: model.yAxisTicks) { value in
                AxisGridLine(stroke: ChartGridStyle.strokeStyle)
                    .foregroundStyle(ChartGridStyle.color)
                AxisTick(stroke: StrokeStyle(lineWidth: ChartGridStyle.lineWidth))
                    .foregroundStyle(ChartGridStyle.color)
                AxisValueLabel {
                    if let price = value.as(Double.self) {
                        Text(model.formattedPrice(price))
                            .font(.caption2)
                            .foregroundStyle(Colors.gray)
                            .padding(.horizontal, .extraSmall)
                    }
                }
            }
            if let currentPrice = model.currentPrice {
                AxisMarks(position: .trailing, values: [currentPrice]) { value in
                    AxisValueLabel {
                        if let price = value.as(Double.self) {
                            Text(model.formattedPrice(price))
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
        }
        .chartXScale(domain: model.xAxisRange)
        .chartYScale(domain: model.yAxisRange)
    }

    @ChartContentBuilder
    private var candlestickMarks: some ChartContent {
        ForEach(model.candles, id: \.date) { candle in
            RuleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.low, candle.low),
                yEnd: .value(ChartKey.high, candle.high),
            )
            .lineStyle(StrokeStyle(lineWidth: .space1))
            .foregroundStyle(model.candleColor(for: candle))

            RectangleMark(
                x: .value(ChartKey.date, candle.date),
                yStart: .value(ChartKey.open, candle.open),
                yEnd: .value(ChartKey.close, candle.close),
                width: .fixed(.space4),
            )
            .foregroundStyle(model.candleColor(for: candle))
        }
    }

    @ChartContentBuilder
    private var linesMarks: some ChartContent {
        ForEach(model.visibleLines) { line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(line.color.opacity(.semiStrong))
                .lineStyle(line.lineStyle)
        }

        ForEach(Array(model.visibleLines.enumerated()), id: \.element.id) { index, line in
            RuleMark(y: .value(ChartKey.price, line.price))
                .foregroundStyle(.clear)
                .annotation(position: .overlay, alignment: .leading, spacing: 0) {
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

    private func vibrate() {
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
}
