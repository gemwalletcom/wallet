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
        try? setChartPeriod(period: period.toGem())
    }

    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        candleSubscription(perpetual: perpetual.toGem(), period: period.toGem())
    }

    func candlesticks(perpetual: Perpetual, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await candlesticks(perpetual: perpetual.toGem(), period: period.toGem()).map { $0.toPrimitives() }
    }

    func mergedCandles(update: ChartCandleUpdate, into candlesticks: [ChartCandleStick], perpetual: Perpetual, period: ChartPeriod) -> [ChartCandleStick]? {
        mergedCandles(candles: candlesticks.map { $0.toGem() }, update: update.toGem(), perpetual: perpetual.toGem(), period: period.toGem())?
            .map { $0.toPrimitives() }
    }
}
