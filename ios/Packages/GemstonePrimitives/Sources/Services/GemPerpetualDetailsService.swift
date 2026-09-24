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

    func mergedCandles(update: ChartCandleUpdate, into candlesticks: [ChartCandleStick], perpetual: Perpetual, period: ChartPeriod) -> [ChartCandleStick]? {
        mergedCandles(candles: candlesticks.map { $0.toGem() }, update: update.toGem(), perpetual: perpetual.toGem(), period: period.toGem())?
            .map { $0.toPrimitives() }
    }
}
