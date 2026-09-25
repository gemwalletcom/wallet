// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.perpetualPositionRows
import GemstonePrimitives
@testable import Perpetuals
import Primitives
import PrimitivesTestKit
import Style
import SwiftUI
import Testing

struct PerpetualPositionItemViewModelTests {
    @Test
    func nameReadsTheAssetSymbol() {
        #expect(model(.mock()).name == "BTC")
    }

    @Test
    func itemsAreKeyedByTheirPosition() {
        let positions: [PerpetualPositionData] = [.mock(position: .mock(id: "1")), .mock(position: .mock(id: "2"))]

        #expect(PerpetualPositionItemViewModel.items(positions, showBalancePrivacy: .constant(false)).map(\.model.id) == ["1", "2"])
    }

    @Test
    func subtitleShoutsTheDirectionAndLeverage() {
        #expect(subtitle(.mock(position: .mock(size: 100, leverage: 5)))?.text == "LONG 5x")
        #expect(subtitle(.mock(position: .mock(size: -100, leverage: 5)))?.text == "SHORT 5x")
    }

    @Test
    func directionColorsTheSubtitle() {
        #expect(subtitle(.mock(position: .mock(direction: .long)))?.style.color == Colors.green)
        #expect(subtitle(.mock(position: .mock(direction: .short)))?.style.color == Colors.red)
    }

    @Test
    func marginAndProfitSitOnTheRight() {
        guard case let .balance(balance, totalFiat) = model(.mock(position: .mock(marginAmount: 12.5, pnl: -1.25))).rightView else {
            Issue.record("a position always shows its margin")
            return
        }
        #expect(balance.text == "$12.50")
        #expect(totalFiat.style.color == Colors.red)
    }

    private func subtitle(_ data: PerpetualPositionData) -> TextValue? {
        guard case let .type(value) = model(data).subtitleView else { return nil }
        return value
    }

    private func model(_ data: PerpetualPositionData) -> PerpetualPositionItemViewModel {
        PerpetualPositionItemViewModel(row: perpetualPositionRows(positions: [data.toGem()])[0])
    }
}
