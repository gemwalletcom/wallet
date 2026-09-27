// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSlippageSelection
import Primitives
import PrimitivesComponents
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct SwapSlippageSceneViewModelTests {
    @Test
    func confirmAuto() {
        var applied: GemSlippageSelection?
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: 50)) { applied = $0 }
        model.isAuto = true
        model.confirm()

        #expect(applied == .auto)
    }

    @Test(arguments: [
        ("1", UInt32(100)),
        ("5", UInt32(500)),
        ("10", UInt32(1000)),
    ] as [(String, UInt32)])
    func confirmAppliesManualValue(input: String, expected: UInt32) {
        var applied: GemSlippageSelection?
        let model = SwapSlippageSceneViewModel.mock { applied = $0 }
        model.isAuto = false
        model.input = input
        model.confirm()

        #expect(model.viewState.selection == .manual(bps: expected))
        #expect(applied == .manual(bps: expected))
    }

    @Test
    func onSelectSuggestionUpdatesInput() {
        let model = SwapSlippageSceneViewModel.mock()
        model.isAuto = false
        model.onSelect(suggestion: model.viewState.suggestions[2])

        #expect(model.input == "3")
        #expect(model.viewState.selection == .manual(bps: 300))
    }
}
