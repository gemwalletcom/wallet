// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.ChartDateValue
import struct Gemstone.GemChartData
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

internal import Charts

public struct ChartView: View {
    private enum ChartKey {
        static let date = "Date"
        static let value = "Value"
    }

    private enum Metrics {
        static let lineWidth: CGFloat = 2.5
        static let selectionDotSize: CGFloat = 12
        static let labelWidth: CGFloat = 88
    }

    private let chart: GemChartData
    private let lineColor: Color
    private let dateFormatter = ChartDateFormatter()
    private let onZoom: @MainActor (Double, Double) -> Void
    private let onPan: @MainActor (Double) -> Void

    @Binding private var isPinching: Bool
    @State private var selectedIndex: Int?
    @State private var valueRange: ClosedRange<Double>

    init(chart: GemChartData, lineColor: Color = Colors.blue, isPinching: Binding<Bool>, onZoom: @escaping @MainActor (Double, Double) -> Void, onPan: @escaping @MainActor (Double) -> Void) {
        self.chart = chart
        self.lineColor = lineColor
        _isPinching = isPinching
        self.onZoom = onZoom
        self.onPan = onPan
        _valueRange = State(initialValue: chart.bounds.yMin ... chart.bounds.yMax)
    }

    public var body: some View {
        VStack(spacing: .zero) {
            priceHeader
            chartView
        }
        .sensoryFeedback(.selection, trigger: selectedElement?.date) { _, date in date != nil }
    }
}

// MARK: - UI

extension ChartView {
    private var priceHeader: some View {
        Group {
            if let selection = selectedIndex.flatMap({ chart.selection(index: UInt32($0)) }) {
                ChartHeaderView(header: selection.header, date: dateFormatter.string(for: selection.date, style: chart.dateStyle))
            } else if let header = chart.header {
                ChartHeaderView(header: header)
            }
        }
        .padding(.top, Spacing.small)
        .padding(.bottom, Spacing.tiny)
    }

    private var selectedElement: ChartDateValue? {
        selectedIndex.flatMap { chart.values[safe: $0] }
    }

    private var chartView: some View {
        Chart {
            ForEach(chart.values, id: \.date) { item in
                AreaMark(
                    x: .value(ChartKey.date, item.date),
                    y: .value(ChartKey.value, item.value),
                )
                .interpolationMethod(.catmullRom)
                .foregroundStyle(areaGradient)
                .alignsMarkStylesWithPlotArea()

                LineMark(
                    x: .value(ChartKey.date, item.date),
                    y: .value(ChartKey.value, item.value),
                )
                .lineStyle(StrokeStyle(lineWidth: Metrics.lineWidth, lineCap: .round, lineJoin: .round))
                .foregroundStyle(lineColor)
                .interpolationMethod(.catmullRom)
            }

            if let selectedElement {
                RuleMark(x: .value(ChartKey.date, selectedElement.date))
                    .foregroundStyle(lineColor.opacity(.medium))
                    .lineStyle(StrokeStyle(lineWidth: 1, dash: [4, 4]))

                PointMark(x: .value(ChartKey.date, selectedElement.date), y: .value(ChartKey.value, selectedElement.value))
                    .symbol {
                        Circle()
                            .fill(
                                RadialGradient(
                                    colors: [Colors.white, lineColor.opacity(.strong)],
                                    center: .center,
                                    startRadius: 0,
                                    endRadius: Metrics.selectionDotSize / 2,
                                ),
                            )
                            .frame(width: Metrics.selectionDotSize, height: Metrics.selectionDotSize)
                            .shadow(color: lineColor.opacity(.semiStrong), radius: 6)
                            .overlay(Circle().strokeBorder(lineColor, lineWidth: Metrics.lineWidth))
                    }
            }
        }
        .chartOverlay { proxy in
            GeometryReader { geometry in
                if let plotFrame = proxy.plotFrame {
                    Color.clear
                        .chartGestures(
                            in: geometry[plotFrame],
                            isPinching: $isPinching,
                            onScrub: { selectedIndex = chart.indexAt(fraction: $0).map(Int.init) },
                            onScrubEnd: { selectedIndex = nil },
                            onZoom: onZoom,
                            onPan: onPan,
                        )
                }

                if let lastPoint = chart.values.last,
                   lastPoint.date <= chart.end,
                   let plotFrame = proxy.plotFrame,
                   let xPos = proxy.position(forX: lastPoint.date),
                   let yPos = proxy.position(forY: lastPoint.value)
                {
                    let origin = geometry[plotFrame].origin
                    PulsingDotView(color: lineColor)
                        .position(x: origin.x + xPos, y: origin.y + yPos)
                        .opacity(selectedElement == nil ? 1 : 0)
                        .animation(.easeInOut(duration: .AnimationDuration.normal), value: selectedElement == nil)
                }
            }
        }
        .padding(.vertical, Spacing.large)
        .chartXAxis(.hidden)
        .chartYAxis(.hidden)
        .chartYScale(domain: valueRange)
        .chartRange($valueRange, fitting: chart.bounds.yMin ... chart.bounds.yMax)
        .chartXScale(domain: chart.start ... chart.end)
        .chartPlotStyle { plotArea in
            plotArea.clipped()
        }
        .chartBackground { proxy in
            GeometryReader { geometry in
                if let plotFrame = proxy.plotFrame {
                    let chartBounds = geometry[plotFrame]

                    if let lowerBoundX = proxy.position(forX: chart.values[Int(chart.bounds.lowerIndex)].date) {
                        boundLabel(chart.bounds.low.text())
                            .offset(x: labelX(lowerBoundX, geoWidth: geometry.size.width), y: chartBounds.maxY + Spacing.small)
                    }

                    if let upperBoundX = proxy.position(forX: chart.values[Int(chart.bounds.upperIndex)].date) {
                        boundLabel(chart.bounds.high.text())
                            .offset(x: labelX(upperBoundX, geoWidth: geometry.size.width), y: chartBounds.minY - Spacing.large)
                    }
                }
            }
        }
    }

    private var areaGradient: LinearGradient {
        .linearGradient(
            stops: [
                .init(color: lineColor.opacity(.opacity45), location: 0),
                .init(color: lineColor.opacity(.opacity38), location: 0.25),
                .init(color: lineColor.opacity(.opacity28), location: 0.5),
                .init(color: lineColor.opacity(.light), location: 0.75),
                .init(color: lineColor.opacity(.faint), location: 0.92),
                .init(color: lineColor.opacity(0), location: 1.0),
            ],
            startPoint: .top,
            endPoint: .bottom,
        )
    }

    private func boundLabel(_ text: String) -> some View {
        Text(text)
            .font(.caption2)
            .foregroundStyle(Colors.gray)
            .frame(width: Metrics.labelWidth)
    }

    private func labelX(_ x: CGFloat, geoWidth: CGFloat) -> CGFloat {
        let half = Metrics.labelWidth / 2
        let minLeading: CGFloat = Spacing.small
        let maxLeading = max(minLeading, geoWidth - Metrics.labelWidth - Spacing.small)
        return min(maxLeading, max(minLeading, x - half))
    }
}
