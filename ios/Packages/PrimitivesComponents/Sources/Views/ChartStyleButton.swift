// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemChartStyle
import SwiftUI

struct ChartStyleButton: View {
    let style: GemChartStyle
    let action: @MainActor (GemChartStyle) -> Void

    var body: some View {
        Button {
            action(style)
        } label: {
            Image(systemName: style.systemImage)
        }
        .accessibilityLabel(style.title)
        .accessibilityIdentifier("chart_style_switch")
    }
}
