// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import class Gemstone.Config
import enum Gemstone.GemSlippageCheck
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
    private static let maxFractionDigits: Int = 2
    private nonisolated static let formatter = NumericFormatter()

    private let service: any GemSwapQuoteServiceProtocol
    private let onSelect: (SwapSlippage) -> Void
    private let suggestionsBps: [UInt32]
    private let maxPercent: Double

    var isAuto: Bool
    var inputModel: InputValidationViewModel
    var infoSheet: InfoSheetType?

    public init(service: any GemSwapQuoteServiceProtocol, chain: Chain, slippage: SwapSlippage, onSelect: @escaping (SwapSlippage) -> Void) {
        self.service = service
        self.onSelect = onSelect
        let config = Config.shared.swapConfig()
        suggestionsBps = config.slippageSuggestionsBps
        maxPercent = Double(config.maxSlippageBps) / 100
        let bps: UInt32
        switch slippage {
        case .auto:
            isAuto = true
            bps = service.defaultSlippage(chain: chain.rawValue).bps
        case let .manual(value):
            isAuto = false
            bps = value
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
        inputModel.text = Self.format(bps: bps)
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
        Self.bps(from: inputModel.text) ?? 0
    }

    var errorText: String? {
        inputModel.error?.localizedDescription
    }

    var isConfirmEnabled: Bool {
        isAuto || (inputModel.isValid && selectedBps > 0)
    }

    var warningText: String? {
        guard inputModel.isValid, service.slippageCheck(bps: selectedBps) == .high else { return nil }
        return Localized.Swap.slippageWarning
    }

    var suggestions: [SlippageSuggestion] {
        suggestionsBps.map { SlippageSuggestion(bps: $0, percentText: Self.format(bps: $0)) }
    }

    func onSelect(suggestion: SlippageSuggestion) {
        inputModel.text = suggestion.inputValue
    }

    func onSelectInfo() {
        infoSheet = .slippage
    }

    func sanitize(_ text: String) -> String {
        NumberSanitizer(
            maximumFractionDigits: Self.maxFractionDigits,
            maximumIntegerDigits: String(Int(maxPercent)).count,
        ).sanitize(text)
    }

    func confirm() {
        onSelect(isAuto ? .auto : .manual(bps: selectedBps))
    }

    nonisolated static func bps(from text: String) -> UInt32? {
        guard let percent = formatter.double(from: text), percent > 0 else { return nil }
        return UInt32((percent * 100).rounded())
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
        guard let bps = SwapSlippageViewModel.bps(from: text) else { return }
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
