// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCandleViewport
import Primitives

public struct PerpetualCandles: Sendable {
    public let period: ChartPeriod
    public let viewport: GemCandleViewport
    public let base: Double
}
