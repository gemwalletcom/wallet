// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.PerpetualPosition
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualChartSection: View {
    @Bindable var chart: PerpetualChartViewModel
    let position: PerpetualPosition?
    let onPeriodChange: @MainActor (ChartPeriod, ChartPeriod) -> Void

    var body: some View {
        VStack {
            VStack {
                switch chart.state(position: position) {
                case .noData:
                    StateEmptyView(title: chart.emptyTitle, image: chart.emptyImage)
                case .loading:
                    LoadingView()
                case let .data(data):
                    CandlestickChartView(chart: data, isPinching: $chart.isPinching, onZoom: chart.onZoom)
                case let .error(error):
                    StateEmptyView(
                        title: error.networkOrNoDataDescription,
                        image: Images.ErrorContent.error,
                    )
                }
            }
            .frame(height: Sizing.chart.height)

            PeriodSelectorView(selectedPeriod: $chart.currentPeriod)
                .padding(.horizontal, Spacing.medium)
        }
        .onChange(of: chart.currentPeriod, onPeriodChange)
    }
}
