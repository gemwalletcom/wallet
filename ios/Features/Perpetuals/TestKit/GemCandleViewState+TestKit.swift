// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemCandleViewport
import struct Gemstone.GemCandleViewState
import GemstonePrimitives
import Primitives

public extension GemCandleViewState {
    static func mock(candles: [ChartCandleStick] = []) -> GemCandleViewState {
        GemCandleViewState(
            period: Primitives.ChartPeriod.day.toGem(),
            state: .data,
            viewport: .mock(candles: candles),
            base: candles.first?.close ?? 0,
            isRefreshing: false,
        )
    }
}
