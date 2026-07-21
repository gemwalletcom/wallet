// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
@testable import Swap
import Testing

@MainActor
struct SwapSlippageViewModelTests {
    @Test
    func initAutoUsesDefaultBps() {
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 300) { _ in }

        #expect(model.isAuto)
        #expect(model.selectedBps == 300)
        #expect(model.inputModel.text == "3")
    }

    @Test
    func initManual() {
        let model = SwapSlippageViewModel(slippage: .manual(bps: 50), defaultBps: 100) { _ in }

        #expect(model.isAuto == false)
        #expect(model.selectedBps == 50)
        #expect(model.inputModel.text == "0.5")
    }

    @Test
    func confirmAuto() {
        var applied: SwapSlippage?
        let model = SwapSlippageViewModel(slippage: .manual(bps: 50), defaultBps: 100) { applied = $0 }
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
        var applied: SwapSlippage?
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 100) { applied = $0 }
        model.isAuto = false
        model.inputModel.text = input
        model.confirm()

        #expect(model.selectedBps == expected)
        #expect(applied == .manual(bps: expected))
    }

    @Test
    func aboveMaximumShowsErrorAndDisablesConfirm() {
        let model = SwapSlippageViewModel(slippage: .manual(bps: 100), defaultBps: 100) { _ in }
        model.isAuto = false
        model.inputModel.text = "25"

        #expect(model.errorText != nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test
    func belowMinimumShowsErrorAndDisablesConfirm() {
        let model = SwapSlippageViewModel(slippage: .manual(bps: 100), defaultBps: 100) { _ in }
        model.isAuto = false
        model.inputModel.text = "0.05"

        #expect(model.errorText != nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test(arguments: ["", "0", "0.", "abc"])
    func incompleteInputDisablesConfirmWithoutError(input: String) {
        let model = SwapSlippageViewModel(slippage: .manual(bps: 100), defaultBps: 100) { _ in }
        model.isAuto = false
        model.inputModel.text = input

        #expect(model.errorText == nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test
    func confirmEnabledState() {
        let model = SwapSlippageViewModel(slippage: .manual(bps: 100), defaultBps: 100) { _ in }
        #expect(model.isConfirmEnabled)

        model.inputModel.text = ""
        #expect(model.errorText == nil)
        #expect(model.isConfirmEnabled == false)

        model.inputModel.text = "5"
        #expect(model.isConfirmEnabled)

        model.isAuto = true
        model.inputModel.text = ""
        #expect(model.isConfirmEnabled)
    }

    @Test
    func suggestionsProvideExpectedValues() {
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 100) { _ in }

        #expect(model.suggestions.map(\.title) == ["0.3%", "0.5%", "3%"])
        #expect(model.suggestions.map(\.inputValue) == ["0.3", "0.5", "3"])
    }

    @Test
    func onSelectSuggestionUpdatesInput() {
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 100) { _ in }
        model.isAuto = false
        model.onSelect(suggestion: model.suggestions[2])

        #expect(model.inputModel.text == "3")
        #expect(model.selectedBps == 300)
    }

    @Test
    func maximumBoundaryIsValid() {
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 100) { _ in }
        model.isAuto = false
        model.inputModel.text = "20"

        #expect(model.inputModel.isValid)
        #expect(model.selectedBps == 2000)
    }

    @Test
    func minimumBoundaryIsValid() {
        let model = SwapSlippageViewModel(slippage: .auto, defaultBps: 100) { _ in }
        model.isAuto = false
        model.inputModel.text = "0.1"

        #expect(model.inputModel.isValid)
        #expect(model.selectedBps == 10)
    }

    @Test(arguments: [
        (UInt32(10), false),
        (UInt32(100), false),
        (UInt32(290), false),
        (UInt32(300), true),
        (UInt32(500), true),
    ] as [(UInt32, Bool)])
    func warning(bps: UInt32, expected: Bool) {
        let model = SwapSlippageViewModel(slippage: .manual(bps: bps), defaultBps: 100) { _ in }

        #expect((model.warningText != nil) == expected)
    }
}
