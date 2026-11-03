// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemCandleChart
import struct Gemstone.GemChartData
import enum Gemstone.GemChartStyle
import Primitives

@MainActor
public protocol ChartListViewable: AnyObject, Observable {
    var chartState: StateViewType<GemChartData> { get }
    var selectedPeriod: ChartPeriod { get set }
    var periods: [ChartPeriod] { get }
    var candleChart: GemCandleChart? { get }
    var chartStyle: GemChartStyle { get }
    var styleSwitch: GemChartStyle? { get }
    func load() async
    func onZoom(_ magnification: Double, anchor: Double)
    func onPan(_ fraction: Double)
    func onSwitchStyle(_ style: GemChartStyle)
}

public extension ChartListViewable {
    var periods: [ChartPeriod] {
        [.hour, .day, .week, .month, .year, .all]
    }

    var candleChart: GemCandleChart? {
        nil
    }

    var chartStyle: GemChartStyle {
        .line
    }

    var styleSwitch: GemChartStyle? {
        nil
    }

    func onSwitchStyle(_: GemChartStyle) {}
}
