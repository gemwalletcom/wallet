// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import Style
import SwiftUI

public struct PeriodSelectorView: View {
    @Binding var selectedPeriod: ChartPeriod
    let periods: [ChartPeriod]

    public init(
        selectedPeriod: Binding<ChartPeriod>,
        periods: [ChartPeriod] = [.hour, .day, .week, .month, .year, .all],
    ) {
        _selectedPeriod = selectedPeriod
        self.periods = periods
    }

    public var body: some View {
        HStack(alignment: .center, spacing: Spacing.medium) {
            ForEach(periods) { period in
                Button {
                    selectedPeriod = period
                } label: {
                    Text(period.title)
                        .fontWeight(.medium)
                        .frame(maxWidth: .infinity)
                        .padding(.space6)
                        .background(selectedPeriod == period ? Colors.white : .clear)
                        .cornerRadius(8)
                }
                .buttonStyle(.borderless)
            }
        }
        .padding(.bottom, Spacing.medium)
    }
}
