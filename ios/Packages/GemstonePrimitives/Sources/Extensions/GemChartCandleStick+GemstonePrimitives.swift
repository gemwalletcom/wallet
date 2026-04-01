// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemChartCandleStick {
    func map() -> ChartCandleStick {
        ChartCandleStick(
            date: Date(timeIntervalSince1970: TimeInterval(date)),
            open: open,
            high: high,
            low: low,
            close: close,
            volume: volume,
        )
    }
}

public extension GemChartCandleUpdate {
    func map() -> ChartCandleUpdate {
        ChartCandleUpdate(
            coin: coin,
            interval: interval,
            candle: candle.map(),
        )
    }
}
