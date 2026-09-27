// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemChartData
import Localization
import Primitives
import Style
import SwiftUI

public struct ChartStateView: View {
    private let state: StateViewType<GemChartData>
    private let periods: [ChartPeriod]

    @Binding private var selectedPeriod: ChartPeriod

    public init(
        state: StateViewType<GemChartData>,
        selectedPeriod: Binding<ChartPeriod>,
        periods: [ChartPeriod] = [.hour, .day, .week, .month, .year, .all],
    ) {
        self.state = state
        _selectedPeriod = selectedPeriod
        self.periods = periods
    }

    public var body: some View {
        VStack {
            VStack {
                switch state {
                case .noData:
                    StateEmptyView(title: Localized.Common.notAvailable, image: Images.EmptyContent.activity)
                case .loading:
                    LoadingView()
                case let .data(chart):
                    ChartView(chart: chart)
                case let .error(error):
                    StateEmptyView(
                        title: error.networkOrNoDataDescription,
                        image: Images.ErrorContent.error,
                    )
                }
            }
            .frame(height: Sizing.chart.height)

            PeriodSelectorView(selectedPeriod: $selectedPeriod, periods: periods)
                .padding(.horizontal, Spacing.medium)
        }
    }
}
