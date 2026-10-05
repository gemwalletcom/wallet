// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.ChartCandleStick
import struct Gemstone.GemCandleChart
import struct Gemstone.GemCandleTooltip
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickChartView: View {
    private let chart: GemCandleChart
    private let dateFormatter = ChartDateFormatter()
    private let onZoom: @MainActor (Double, Double) -> Void
    private let onPan: @MainActor (Double) -> Void

    @ScaledMetric(relativeTo: .caption2) private var priceColumnWidth: CGFloat = 72
    @ScaledMetric(relativeTo: .caption2) private var timeRowHeight: CGFloat = 20

    @Binding private var isPinching: Bool
    @State private var selectedIndex: Int?
    @State private var priceRange: ClosedRange<Double>
    @State private var volumeRange: ClosedRange<Double>

    init(chart: GemCandleChart, isPinching: Binding<Bool>, onZoom: @escaping @MainActor (Double, Double) -> Void, onPan: @escaping @MainActor (Double) -> Void) {
        self.chart = chart
        _isPinching = isPinching
        self.onZoom = onZoom
        self.onPan = onPan
        _priceRange = State(initialValue: chart.layout.priceLow ... chart.layout.priceHigh)
        _volumeRange = State(initialValue: 0 ... chart.layout.volumeHigh)
    }

    var body: some View {
        VStack {
            priceHeader
            chartView
                .padding(.bottom, Spacing.small)
        }
        .sensoryFeedback(.selection, trigger: selectedCandle?.date) { previous, date in previous != nil && date != nil }
    }
}

// MARK: - UI

extension CandlestickChartView {
    private var selectedCandle: ChartCandleStick? {
        selectedIndex.flatMap { chart.candles[safe: $0] }
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
        GeometryReader { geometry in
            let plot = CandlestickPlot(chart: chart, size: geometry.size, priceColumnWidth: priceColumnWidth, timeRowHeight: timeRowHeight, priceRange: priceRange, volumeRange: volumeRange)
            CandlestickCanvas(plot: plot, selectedCandle: selectedCandle)
                .chartGestures(
                    in: plot.frame,
                    isZoomed: chart.isZoomed,
                    isPinching: $isPinching,
                    onScrub: { selectedIndex = chart.indexAt(fraction: $0).map(Int.init) },
                    onScrubEnd: { selectedIndex = nil },
                    onZoom: onZoom,
                    onPan: onPan,
                )
                .overlay {
                    if let selectedCandle, let tooltip = selectedIndex.flatMap({ chart.tooltip(index: UInt32($0)) }) {
                        tooltipOverlay(tooltip, isRightHalf: plot.x(for: selectedCandle.date) > plot.frame.midX)
                    }
                }
        }
        .chartRange($priceRange, fitting: chart.layout.priceLow ... chart.layout.priceHigh)
        .chartRange($volumeRange, fitting: 0 ... chart.layout.volumeHigh)
    }

    private func tooltipOverlay(_ tooltip: GemCandleTooltip, isRightHalf: Bool) -> some View {
        CandleTooltipView(tooltip: tooltip)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: isRightHalf ? .topLeading : .topTrailing)
            .padding(.leading, Spacing.small)
            .padding(.top, Spacing.small)
            .padding(.trailing, priceColumnWidth + Spacing.small)
            .transition(.opacity)
            .animation(.easeInOut(duration: Interval.AnimationDuration.fast), value: isRightHalf)
            .allowsHitTesting(false)
    }
}
