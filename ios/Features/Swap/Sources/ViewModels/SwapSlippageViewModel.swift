// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import class Gemstone.Config
import struct Gemstone.GemNumberFormat
import enum Gemstone.GemSlippageCheck
import enum Gemstone.GemSlippageSelection
import struct Gemstone.GemSlippageSession
import struct Gemstone.GemSlippageViewState
import protocol Gemstone.GemSwapQuoteServiceProtocol
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Validators

@MainActor
@Observable
public final class SwapSlippageViewModel {

    private let service: any GemSwapQuoteServiceProtocol
    private let onSelect: (SwapSlippage) -> Void

    let placeholder: String
    var isAuto: Bool
    var inputModel: InputValidationViewModel
    var infoSheet: InfoSheetType?

    public init(service: any GemSwapQuoteServiceProtocol, chain: Chain, slippage: SwapSlippage, onSelect: @escaping (SwapSlippage) -> Void) {
        self.service = service
        self.onSelect = onSelect
        let config = Config.shared.swapConfig()
        placeholder = Self.format(bps: service.defaultSlippage(chain: chain.rawValue).bps)
        let input: String
        switch slippage {
        case .auto:
            isAuto = true
            input = ""
        case let .manual(value):
            isAuto = false
            input = Self.format(bps: value)
        }
        inputModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [
                SwapSlippageValidator(
                    service: service,
                    minimumText: Self.format(bps: config.minSlippageBps),
                    maximumText: Self.format(bps: config.maxSlippageBps),
                ),
            ],
        )
        inputModel.text = input
    }

    var title: String {
        Localized.Swap.slippage
    }

    var autoTitle: String {
        Localized.Swap.slippageAuto
    }

    var autoDescription: String {
        Localized.Swap.slippageAutoDescription
    }

    var selectedBps: UInt32 {
        Self.bps(from: inputModel.text, service: service) ?? 0
    }

    var errorText: String? {
        inputModel.error?.localizedDescription
    }

    private var viewState: GemSlippageViewState {
        service.newSlippageSession(selection: isAuto ? .auto : .manual(bps: selectedBps)).viewState()
    }

    var isConfirmEnabled: Bool {
        viewState.allowsConfirm
    }

    var warningText: String? {
        guard inputModel.isValid, viewState.showsWarning else { return nil }
        return Localized.Swap.slippageWarning
    }

    var suggestions: [SlippageSuggestion] {
        viewState.suggestionsBps.map { SlippageSuggestion(bps: $0, percentText: Self.format(bps: $0)) }
    }

    func onSelect(suggestion: SlippageSuggestion) {
        inputModel.text = suggestion.inputValue
    }

    func onSelectInfo() {
        infoSheet = .slippage
    }

    func sanitize(_ text: String) -> String {
        let state = viewState
        return NumberInput.format().sanitize(
            input: text,
            maximumFractionDigits: state.maximumFractionDigits,
            maximumIntegerDigits: state.maximumIntegerDigits,
        )
    }

    func confirm() {
        onSelect(isAuto ? .auto : .manual(bps: selectedBps))
    }

    nonisolated static func bps(from text: String, service: any GemSwapQuoteServiceProtocol) -> UInt32? {
        guard let percent = NumberInput.double(text) else { return nil }
        return service.slippageBpsFromPercent(percent: percent)
    }

    private static func format(bps: UInt32) -> String {
        (Double(bps) / 100).formatted(.number.precision(.fractionLength(0 ... 2)))
    }
}

private struct SwapSlippageValidator: TextValidator {
    private let service: any GemSwapQuoteServiceProtocol
    private let minimumText: String
    private let maximumText: String

    init(service: any GemSwapQuoteServiceProtocol, minimumText: String, maximumText: String) {
        self.service = service
        self.minimumText = minimumText
        self.maximumText = maximumText
    }

    func validate(_ text: String) throws {
        guard let bps = SwapSlippageViewModel.bps(from: text, service: service) else { return }
        switch service.slippageCheck(bps: bps) {
        case .valid, .high: return
        case .belowMinimum: throw AnyError(Localized.Common.minimumValue("\(minimumText)%"))
        case .aboveMaximum: throw AnyError(Localized.Common.maximumValue("\(maximumText)%"))
        }
    }

    var id: String {
        "SwapSlippageValidator"
    }
}
