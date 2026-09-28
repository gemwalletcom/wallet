// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

struct ChartStateView<Model: ChartListViewable>: View {
    @Bindable private var model: Model

    init(model: Model) {
        self.model = model
    }

    var body: some View {
        VStack {
            VStack {
                switch model.chartState {
                case .noData:
                    StateEmptyView(title: Localized.Common.notAvailable, image: Images.EmptyContent.activity)
                case .loading:
                    LoadingView()
                case let .data(chart):
                    ChartView(chart: chart, isPinching: $model.isPinching, onZoom: model.onZoom)
                case let .error(error):
                    StateEmptyView(
                        title: error.networkOrNoDataDescription,
                        image: Images.ErrorContent.error,
                    )
                }
            }
            .frame(height: Sizing.chart.height)

            PeriodSelectorView(selectedPeriod: $model.selectedPeriod, periods: model.periods)
                .padding(.horizontal, Spacing.medium)
        }
    }
}
