// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import Testing

struct ChartBoundsTests {
    @Test
    func axisTicksRenderExpectedLabelsAcrossPriceMagnitudes() {
        #expect(labels(for: .mock(open: 76_000, high: 80_000, low: 75_000, close: 77_500)) == ["75000", "76000", "77000", "78000", "79000", "80000"])
        #expect(labels(for: .mock(open: 2_450, high: 2_600, low: 2_400, close: 2_500)) == ["2400", "2450", "2500", "2550", "2600"])
        #expect(labels(for: .mock(open: 150, high: 200, low: 140, close: 165)) == ["140.00", "150.00", "160.00", "170.00", "180.00", "190.00", "200.00"])
        #expect(labels(for: .mock(open: 41, high: 46, low: 34, close: 41)) == ["35.00", "37.50", "40.00", "42.50", "45.00"])
        #expect(labels(for: .mock(open: 0.09, high: 0.12, low: 0.08, close: 0.11)) == ["0.08", "0.09", "0.10", "0.11", "0.12"])
        #expect(labels(for: .mock(open: 0.126, high: 0.140, low: 0.120, close: 0.130)) == ["0.120", "0.125", "0.130", "0.135", "0.140"])
        #expect(labels(for: .mock(open: 0.0038, high: 0.0040, low: 0.0037, close: 0.0039)) == ["0.00370", "0.00375", "0.00380", "0.00385", "0.00390", "0.00395", "0.00400"])
        #expect(labels(for: .mock(open: 0.0000100, high: 0.0000115, low: 0.0000095, close: 0.0000110)) == ["0.0000095", "0.0000100", "0.0000105", "0.0000110", "0.0000115"])
    }

    @Test
    func axisFractionLengthFallsBackGracefullyForFlatRange() {
        let bounds = ChartBounds(
            candles: [.mock(open: 100, high: 100, low: 100, close: 100)],
            lines: [],
        )

        #expect(bounds.axisFractionLength == 2)
        #expect(bounds.axisTicks.isEmpty)
    }

    private func labels(for candle: ChartCandleStick) -> [String] {
        let bounds = ChartBounds(candles: [candle], lines: [])
        return bounds.axisTicks.map {
            String(format: "%.\(bounds.axisFractionLength)f", $0)
        }
    }
}
