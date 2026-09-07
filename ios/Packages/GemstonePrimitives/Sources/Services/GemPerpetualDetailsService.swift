// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import Primitives

public extension GemPerpetualDetailsServiceProtocol {
    var chartPeriodValue: ChartPeriod {
        chartPeriod().map()
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        try? setChartPeriod(period: period.map())
    }

    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        candleSubscription(perpetual: perpetual.map(), period: period.map())
    }

    func candlesticks(perpetual: Perpetual, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await candlesticks(perpetual: perpetual.map(), period: period.map()).map { $0.map() }
    }

    func apply(update: ChartCandleUpdate, to candlesticks: [ChartCandleStick], perpetual: Perpetual, period: ChartPeriod) -> [ChartCandleStick]? {
        applyCandleUpdate(candles: candlesticks.map { $0.map() }, update: update.map(), perpetual: perpetual.map(), period: period.map())?
            .map { $0.map() }
    }
}
