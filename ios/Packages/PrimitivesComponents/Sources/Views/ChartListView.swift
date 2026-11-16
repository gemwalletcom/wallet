// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemChartData
import enum Gemstone.GemChartStyle
import Primitives
import SwiftUI

public struct ChartListView<Model: ChartListViewable, Content: View>: View {
    @Environment(\.connectionStatus) private var connectionStatus

    let model: Model
    @ViewBuilder let content: () -> Content

    @State private var isPinching = false

    public init(model: Model, @ViewBuilder content: @escaping () -> Content) {
        self.model = model
        self.content = content
    }

    public var body: some View {
        List {
            Section {} header: {
                ChartListHeader(model: model, isPinching: $isPinching)
            }
            .fullWidthSection()
            content()
        }
        .scrollDisabled(isPinching)
        .listSectionSpacing(.compact)
        .background {
            ChartPeriodLoader(model: model)
        }
        .refreshable {
            await model.load()
        }
        .refreshableTimer(every: connectionStatus.refreshInterval(for: .chart)) { @MainActor _ in
            await model.load()
        }
        .toolbar {
            if let styleSwitch = model.styleSwitch {
                ToolbarItem(placement: .topBarTrailing) {
                    ChartStyleButton(style: styleSwitch, action: model.onSwitchStyle)
                }
            }
        }
    }
}

private struct ChartListHeader<Model: ChartListViewable>: View {
    @Bindable var model: Model
    @Binding var isPinching: Bool

    var body: some View {
        switch model.chartPhase {
        case .loading: lineChart(.loading)
        case let .data(data): lineChart(.data(data))
        case let .candles(candleChart):
            ChartStateView(state: .data(candleChart), selectedPeriod: $model.selectedPeriod, periods: model.periods) { chart in
                CandlestickChartView(chart: chart, isPinching: $isPinching, onZoom: model.onZoom, onPan: model.onPan)
            }
        case .noData: lineChart(.noData)
        case let .failed(error): lineChart(.error(error))
        }
    }

    private func lineChart(_ state: StateViewType<GemChartData>) -> some View {
        ChartStateView(state: state, selectedPeriod: $model.selectedPeriod, periods: model.periods) { chart in
            ChartView(chart: chart, isPinching: $isPinching, onZoom: model.onZoom, onPan: model.onPan)
        }
    }
}

private struct ChartPeriodLoader<Model: ChartListViewable>: View {
    let model: Model

    var body: some View {
        Color.clear
            .task(id: ChartLoadID(period: model.selectedPeriod, style: model.chartStyle)) {
                await model.load()
            }
    }
}

private struct ChartLoadID: Hashable {
    let period: ChartPeriod
    let style: GemChartStyle
}
