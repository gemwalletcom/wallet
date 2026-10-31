// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemChartStyle
import Localization
import Style
import SwiftUI

struct ChartStyleButton: View {
    let style: GemChartStyle
    let action: @MainActor (GemChartStyle) -> Void

    var body: some View {
        Button {
            action(style)
        } label: {
            Image(systemName: systemImage)
                .fontWeight(.medium)
                .padding(.space6)
        }
        .buttonStyle(.borderless)
        .accessibilityLabel(title)
        .accessibilityIdentifier("chart_style_switch")
    }

    private var systemImage: String {
        switch style {
        case .line: SystemImage.chartXyaxisLine
        case .candles: SystemImage.chartBarXaxis
        }
    }

    private var title: String {
        switch style {
        case .line: Localized.Charts.line
        case .candles: Localized.Charts.candlestick
        }
    }
}
