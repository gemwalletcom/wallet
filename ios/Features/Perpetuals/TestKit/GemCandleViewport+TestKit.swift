// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemCandleViewport
import GemstonePrimitives
import Primitives

public extension GemCandleViewport {
    static func mock(candles: [ChartCandleStick] = []) -> GemCandleViewport {
        GemCandleViewport(
            start: candles.first?.date ?? Date(timeIntervalSince1970: 0),
            end: candles.last?.date ?? Date(timeIntervalSince1970: 0),
            intervalSeconds: 60,
            candles: candles.map { $0.toGem() },
        )
    }
}
