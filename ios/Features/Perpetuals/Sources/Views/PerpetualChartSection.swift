// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.PerpetualPosition
import Primitives
import PrimitivesComponents
import SwiftUI

struct PerpetualChartSection: View {
    @Bindable var chart: PerpetualChartViewModel
    let position: PerpetualPosition?
    let onPeriodChange: @MainActor (ChartPeriod, ChartPeriod) -> Void

    var body: some View {
        ChartStateView(state: chart.state(position: position), selectedPeriod: $chart.currentPeriod) { data in
            CandlestickChartView(chart: data, isPinching: $chart.isPinching, onZoom: chart.onZoom, onPan: chart.onPan)
        }
        .onChange(of: chart.currentPeriod, onPeriodChange)
    }
}
