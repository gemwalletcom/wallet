// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
    }
}

private struct ChartListHeader<Model: ChartListViewable>: View {
    @Bindable var model: Model
    @Binding var isPinching: Bool

    var body: some View {
        if let candleChart = model.candleChart {
            ChartStateView(state: .data(candleChart), selectedPeriod: $model.selectedPeriod, periods: model.periods, styleSwitch: model.styleSwitch, onSwitchStyle: model.onSwitchStyle) { chart in
                CandlestickChartView(chart: chart, isPinching: $isPinching, onZoom: model.onZoom, onPan: model.onPan)
            }
        } else {
            ChartStateView(state: model.chartState, selectedPeriod: $model.selectedPeriod, periods: model.periods, styleSwitch: model.styleSwitch, onSwitchStyle: model.onSwitchStyle) { chart in
                ChartView(chart: chart, isPinching: $isPinching, onZoom: model.onZoom, onPan: model.onPan)
            }
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
