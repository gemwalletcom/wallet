// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSlippageCheck
import GemstoneServicesTestKit
import Primitives
import PrimitivesComponents
@testable import Swap
import Testing

@MainActor
struct SwapSlippageViewModelTests {
    @Test
    func initAuto() {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .auto) { _ in }

        #expect(model.isAuto)
        #expect(model.inputModel.text.isEmpty)
        #expect(model.placeholder == "1")

        model.isAuto = false

        #expect(model.inputModel.text.isEmpty)
        #expect(model.isConfirmEnabled == false)
        #expect(model.errorText == nil)
        #expect(model.warningText == nil)
    }

    @Test
    func initManual() {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .manual(bps: 50)) { _ in }

        #expect(model.isAuto == false)
        #expect(model.selectedBps == 50)
        #expect(model.inputModel.text == "0.5")
    }

    @Test
    func confirmAuto() {
        var applied: SwapSlippage?
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .manual(bps: 50)) { applied = $0 }
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
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .auto) { applied = $0 }
        model.isAuto = false
        model.inputModel.text = input
        model.confirm()

        #expect(model.selectedBps == expected)
        #expect(applied == .manual(bps: expected))
    }

    @Test(arguments: [GemSlippageCheck.aboveMaximum, .belowMinimum])
    func rejectedCheckShowsErrorAndDisablesConfirm(check: GemSlippageCheck) {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(slippageCheck: check), chain: .ethereum, slippage: .manual(bps: 100)) { _ in }
        model.isAuto = false
        model.inputModel.text = "5"

        #expect(model.errorText != nil)
        #expect(model.warningText == nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test(arguments: ["", "0", "0.", "abc"])
    func incompleteInputDisablesConfirmWithoutError(input: String) {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .manual(bps: 100)) { _ in }
        model.isAuto = false
        model.inputModel.text = input

        #expect(model.errorText == nil)
        #expect(model.isConfirmEnabled == false)
    }

    @Test
    func confirmEnabledState() {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .manual(bps: 100)) { _ in }
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
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .auto) { _ in }

        #expect(model.suggestions.map(\.title) == ["0.3%", "0.5%", "3%"])
        #expect(model.suggestions.map(\.inputValue) == ["0.3", "0.5", "3"])
    }

    @Test
    func onSelectSuggestionUpdatesInput() {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(), chain: .ethereum, slippage: .auto) { _ in }
        model.isAuto = false
        model.onSelect(suggestion: model.suggestions[2])

        #expect(model.inputModel.text == "3")
        #expect(model.selectedBps == 300)
    }

    @Test(arguments: [
        (GemSlippageCheck.valid, false),
        (GemSlippageCheck.high, true),
    ] as [(GemSlippageCheck, Bool)])
    func highCheckWarnsButKeepsConfirmEnabled(check: GemSlippageCheck, expected: Bool) {
        let model = SwapSlippageViewModel(service: GemSwapQuoteServiceMock(slippageCheck: check), chain: .ethereum, slippage: .manual(bps: 300)) { _ in }

        #expect((model.warningText != nil) == expected)
        #expect(model.errorText == nil)
        #expect(model.isConfirmEnabled)
    }
}
