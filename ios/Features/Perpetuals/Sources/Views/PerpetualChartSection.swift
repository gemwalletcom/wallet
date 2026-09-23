// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualChartSection: View {
    @Bindable var chart: PerpetualChartModel
    let position: PerpetualPosition?
    let onPeriodChange: @MainActor (ChartPeriod, ChartPeriod) -> Void

    var body: some View {
        VStack {
            VStack {
                switch chart.state {
                case .noData:
                    StateEmptyView(title: chart.emptyTitle, image: chart.emptyImage)
                case .loading: LoadingView()
                case let .data(data):
                    CandlestickChartView(
                        model: CandlestickChartViewModel(
                            viewport: data.viewport,
                            base: data.base,
                            period: data.period,
                            position: position,
                        ),
                        onZoom: chart.onZoom,
                    )
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
