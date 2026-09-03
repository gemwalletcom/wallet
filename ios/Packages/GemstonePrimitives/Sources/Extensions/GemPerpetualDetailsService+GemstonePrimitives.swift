// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualDetailsServiceProtocol
import enum Gemstone.GemPerpetualSubscription
import Primitives

public extension GemPerpetualDetailsServiceProtocol {
    var chartPeriodValue: ChartPeriod {
        (try? ChartPeriod(chartPeriod())) ?? .day
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        try? setChartPeriod(period: period.json())
    }

    func candleSubscription(perpetual: Perpetual, period: ChartPeriod) -> GemPerpetualSubscription {
        candleSubscription(perpetual: perpetual.json(), period: period.json())
    }

    func candlesticks(perpetual: Perpetual, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await candlesticks(perpetual: perpetual.json(), period: period.json()).map { try ChartCandleStick($0) }
    }

    func apply(update: ChartCandleUpdate, to candlesticks: [ChartCandleStick], perpetual: Perpetual, period: ChartPeriod) throws -> [ChartCandleStick]? {
        try applyCandleUpdate(candles: candlesticks.map { $0.json() }, update: update.json(), perpetual: perpetual.json(), period: period.json())?
            .map { try ChartCandleStick($0) }
    }
}
