// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public struct PerpetualCandles: Sendable {
    public let period: ChartPeriod
    public let candles: [ChartCandleStick]
}
