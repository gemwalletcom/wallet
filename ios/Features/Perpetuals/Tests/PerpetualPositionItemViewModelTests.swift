// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Perpetuals
import Primitives
import PrimitivesTestKit
import Style
import SwiftUI
import Testing

struct PerpetualPositionItemViewModelTests {
    @Test
    func nameReadsTheAssetSymbol() {
        #expect(PerpetualPositionItemViewModel(data: .mock()).name == "BTC")
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
        guard case let .balance(balance, totalFiat) = PerpetualPositionItemViewModel(data: .mock(position: .mock(marginAmount: 12.5, pnl: -1.25))).rightView else {
            Issue.record("a position always shows its margin")
            return
        }
        #expect(balance.text == "$12.50")
        #expect(totalFiat.style.color == Colors.red)
    }

    private func subtitle(_ data: PerpetualPositionData) -> TextValue? {
        guard case let .type(value) = PerpetualPositionItemViewModel(data: data).subtitleView else { return nil }
        return value
    }
}
