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
    func initAuto() {
        let model = SwapSlippageSceneViewModel.mock()

        #expect(model.isAuto)
        #expect(model.input.isEmpty)
        #expect(model.placeholder == model.viewState.placeholder)
        #expect(model.placeholder.isEmpty == false)

        model.isAuto = false

        #expect(model.input.isEmpty)
        #expect(model.isConfirmEnabled == false)
        #expect(model.footerText == nil)
    }

    @Test
    func initManual() {
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: 50))

        #expect(model.isAuto == false)
        #expect(model.viewState.selection == .manual(bps: 50))
        #expect(model.input == "0.5")
    }

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

    @Test(arguments: ["25", "0.05"])
    func rejectedInputShowsErrorAndDisablesConfirm(input: String) {
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: 100))
        model.isAuto = false
        model.input = input

        #expect(model.footerText != nil)
        #expect(model.viewState.footer != .warning)
        #expect(model.isConfirmEnabled == false)
    }

    @Test(arguments: ["", "0", "0.", "abc"])
    func incompleteInputDisablesConfirmWithoutError(input: String) {
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: 100))
        model.isAuto = false
        model.input = input

        #expect(model.footerText == nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test
    func confirmEnabledState() {
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: 100))
        #expect(model.isConfirmEnabled)

        model.input = "5"
        #expect(model.isConfirmEnabled)

        model.isAuto = true
        model.input = ""
        #expect(model.isConfirmEnabled)
    }

    @Test
    func suggestionsProvideExpectedValues() {
        let model = SwapSlippageSceneViewModel.mock()

        #expect(model.suggestions.map(\.title) == ["0.3%", "0.5%", "3%"])
        #expect(model.suggestions.map(\.inputValue) == ["0.3", "0.5", "3"])
    }

    @Test
    func onSelectSuggestionUpdatesInput() {
        let model = SwapSlippageSceneViewModel.mock()
        model.isAuto = false
        model.onSelect(suggestion: model.suggestions[2])

        #expect(model.input == "3")
        #expect(model.viewState.selection == .manual(bps: 300))
    }

    @Test(arguments: [
        (UInt32(100), false),
        (UInt32(300), true),
    ] as [(UInt32, Bool)])
    func highSlippageWarnsButKeepsConfirmEnabled(bps: UInt32, expected: Bool) {
        let model = SwapSlippageSceneViewModel.mock(slippage: .manual(bps: bps))

        #expect((model.viewState.footer == .warning) == expected)
        #expect(model.isConfirmEnabled)
    }
}
