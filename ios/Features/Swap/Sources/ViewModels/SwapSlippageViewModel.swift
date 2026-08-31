// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.Config
import Formatters
import Foundation
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
    private static let formatter = NumericFormatter()

    private let onSelect: (SwapSlippage) -> Void
    private let highWarningBps: UInt32
    private let suggestionsBps: [UInt32]
    private let minPercent: Double
    private let maxPercent: Double

    var isAuto: Bool
    var inputModel: InputValidationViewModel
    var infoSheet: InfoSheetType?

    public init(slippage: SwapSlippage, onSelect: @escaping (SwapSlippage) -> Void) {
        self.onSelect = onSelect
        let config = Config.shared.swapConfig()
        highWarningBps = config.highSlippageWarningBps
        suggestionsBps = config.slippageSuggestionsBps
        minPercent = Double(config.minSlippageBps) / 100
        maxPercent = Double(config.maxSlippageBps) / 100
        let bps: UInt32
        switch slippage {
        case .auto:
            isAuto = true
            bps = config.defaultSlippage.bps
        case let .manual(value):
            isAuto = false
            bps = value
        }
        inputModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [PercentTextValidator(minimum: minPercent, maximum: maxPercent)],
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
        parseBps(inputModel.text)
    }

    var errorText: String? {
        inputModel.error?.localizedDescription
    }

    var isConfirmEnabled: Bool {
        isAuto || (inputModel.isValid && selectedBps > 0)
    }

    var warningText: String? {
        guard inputModel.isValid, selectedBps >= highWarningBps else { return nil }
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

    private static func format(bps: UInt32) -> String {
        (Double(bps) / 100).formatted(.number.precision(.fractionLength(0 ... 2)))
    }

    private func parseBps(_ text: String) -> UInt32 {
        guard let percent = Self.formatter.double(from: text), percent > 0 else { return 0 }
        return UInt32((min(percent, maxPercent) * 100).rounded())
    }
}
