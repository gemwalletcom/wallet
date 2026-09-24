// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import Primitives

public extension GemPerpetualDetailsServiceProtocol {
    var chartPeriodValue: ChartPeriod {
        chartPeriod().toPrimitives()
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        do {
            try setChartPeriod(period: period.toGem())
        } catch {
            debugLog("storing the chart period failed: \(error)")
        }
    }

    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        candleSubscription(perpetual: perpetual.toGem(), period: period.toGem())
    }
}
