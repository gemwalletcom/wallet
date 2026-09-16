// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemCandleTooltipRow
import Localization
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct CandleTooltipViewModelTests {
    @Test
    func tooltipContent() {
        let model = CandleTooltipViewModel.mock(candle: .mock(open: 67715, high: 68181, low: 67714, close: 68087, volume: 500))
        let fields = (model.priceCells + model.summaryCells).map { ($0.row, model.field(for: $0)) }

        #expect(fields.map(\.0) == [.open, .high, .low, .close, .change, .volume])
        #expect(fields.map(\.1.title.text) == [
            Localized.Charts.Price.open,
            Localized.Charts.Price.high,
            Localized.Charts.Price.low,
            Localized.Charts.Price.close,
            Localized.Charts.Price.change,
            Localized.Perpetual.volume,
        ])
        #expect(fields.map(\.1.value.text) == ["67,715.00", "68,181.00", "67,714.00", "68,087.00", "+0.55%", "$34.04M"])
    }

    @Test
    func changeSign() {
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 105)).changeText == "+5.00%")
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 95)).changeText == "-5.00%")
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 100)).changeText == "+0.00%")
    }
}

private extension CandleTooltipViewModel {
    var changeText: String? {
        summaryCells.first { $0.row == .change }.map { field(for: $0).value.text }
    }
}
