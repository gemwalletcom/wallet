// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAutocloseSummary
import GemstonePrimitivesTestKit
import Localization
import Testing
@testable import Transfer

struct PerpetualModifyViewModelTests {
    @Test
    func noSummaryDrawsNoRow() {
        #expect(PerpetualModifyViewModel(summary: nil).listItemModel == nil)
    }

    @Test
    func bothTriggersTakeTheirOwnLine() throws {
        let model = PerpetualModifyViewModel(summary: GemAutocloseSummary(
            takeProfit: .mock(value: 120),
            stopLoss: .mock(value: 80),
            takeProfitCleared: false,
            stopLossCleared: false,
        ))
        let row = try #require(model.listItemModel)

        #expect(row.title == Localized.Perpetual.autoClose)
        #expect(row.subtitle?.contains(Localized.Perpetual.takeProfit) == true)
        #expect(row.subtitleExtra?.contains(Localized.Perpetual.stopLoss) == true)
    }

    @Test
    func aClearedTriggerStillNamesItself() throws {
        let model = PerpetualModifyViewModel(summary: GemAutocloseSummary(
            takeProfit: nil,
            stopLoss: nil,
            takeProfitCleared: true,
            stopLossCleared: false,
        ))
        let row = try #require(model.listItemModel)

        #expect(row.subtitle?.contains(Localized.Perpetual.takeProfit) == true)
        #expect(row.subtitleExtra == nil)
    }
}
