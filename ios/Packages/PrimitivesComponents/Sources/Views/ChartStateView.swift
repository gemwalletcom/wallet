// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct ChartStateView<Chart: Sendable, Content: View>: View {
    private let state: StateViewType<Chart>
    private let periods: [ChartPeriod]
    private let content: (Chart) -> Content

    @Binding private var selectedPeriod: ChartPeriod

    public init(
        state: StateViewType<Chart>,
        selectedPeriod: Binding<ChartPeriod>,
        periods: [ChartPeriod] = [.hour, .day, .week, .month, .year, .all],
        @ViewBuilder content: @escaping (Chart) -> Content,
    ) {
        self.state = state
        _selectedPeriod = selectedPeriod
        self.periods = periods
        self.content = content
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
                    content(chart)
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
