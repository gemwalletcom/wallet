// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemChartPhase
import enum Gemstone.GemChartStyle
import Primitives

@MainActor
public protocol ChartListViewable: AnyObject, Observable {
    var chartPhase: GemChartPhase { get }
    var selectedPeriod: ChartPeriod { get set }
    var periods: [ChartPeriod] { get }
    var chartStyle: GemChartStyle { get }
    var styleSwitch: GemChartStyle? { get }
    func load() async
    func onZoom(_ magnification: Double, anchor: Double)
    func onPan(_ fraction: Double)
    func onSwitchStyle(_ style: GemChartStyle)
}

public extension ChartListViewable {
    var chartStyle: GemChartStyle {
        .line
    }

    var styleSwitch: GemChartStyle? {
        nil
    }

    func onSwitchStyle(_: GemChartStyle) {}
}
