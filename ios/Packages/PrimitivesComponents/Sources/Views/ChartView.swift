// Copyright (c). Gem Wallet. All rights reserved.

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
        static let trailingSpace: Double = 0.02
    }

    private let chart: GemChartData
    private let lineColor: Color
    private let dateFormatter = ChartDateFormatter()

    @State private var selectedIndex: Int?

    public init(chart: GemChartData, lineColor: Color = Colors.blue) {
        self.chart = chart
        self.lineColor = lineColor
    }

    public var body: some View {
        VStack(spacing: .zero) {
            priceHeader
            chartView
        }
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

    private var yScale: [Double] {
        [chart.bounds.yMin, chart.bounds.yMax]
    }

    private var xScale: [Date] {
        guard let first = chart.values.first?.date, let last = chart.values.last?.date else { return [] }
        return [first, last.addingTimeInterval(last.timeIntervalSince(first) * Metrics.trailingSpace)]
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
                Rectangle()
                    .fill(.clear)
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 0)
                            .onChanged { value in
                                onDragChange(location: value.location, proxy: proxy, geometry: geometry)
                            }
                            .onEnded { _ in
                                onDragEnd()
                            },
                    )

                if let lastPoint = chart.values.last,
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
        .chartYScale(domain: yScale)
        .chartXScale(domain: xScale)
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

// MARK: - Actions

extension ChartView {
    private func onDragChange(location: CGPoint, proxy: ChartProxy, geometry: GeometryProxy) {
        guard let plotFrame = proxy.plotFrame else { return }

        let relativeX = location.x - geometry[plotFrame].origin.x
        guard let targetDate = proxy.value(atX: relativeX) as Date?,
              let index = chart.values.indices.min(by: { abs(chart.values[$0].date.distance(to: targetDate)) < abs(chart.values[$1].date.distance(to: targetDate)) })
        else {
            return
        }

        selectedIndex = index
    }

    private func onDragEnd() {
        selectedIndex = nil
    }
}
