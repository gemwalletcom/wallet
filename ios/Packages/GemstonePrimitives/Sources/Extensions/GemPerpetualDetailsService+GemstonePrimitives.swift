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

    func candleSubscription(symbol: String, period: ChartPeriod) -> GemPerpetualSubscription {
        candleSubscription(symbol: symbol, period: period.json())
    }

    func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await candlesticks(symbol: symbol, period: period.json()).map { try ChartCandleStick($0) }
    }

    func merge(candlesticks: [ChartCandleStick], candle: ChartCandleStick) throws -> [ChartCandleStick] {
        try mergeCandle(candles: candlesticks.map { $0.json() }, candle: candle.json()).map { try ChartCandleStick($0) }
    }
}
