// Copyright (c). Gem Wallet. All rights reserved.

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
    private static let defaultBps: UInt32 = 100
    private static let suggestionsBps: [UInt32] = [30, 50, 300]
    static let minPercent: Double = 0.1
    static let maxPercent: Double = 20
    private static let maxFractionDigits: Int = 2
    private static let maxIntegerDigits: Int = String(Int(maxPercent)).count
    private static let formatter = NumericFormatter()

    private let onSelect: (SwapSlippage) -> Void
    private let highWarningBps: UInt32

    var isAuto: Bool
    var inputModel: InputValidationViewModel
    var infoSheet: InfoSheetType?

    public init(slippage: SwapSlippage, onSelect: @escaping (SwapSlippage) -> Void) {
        self.onSelect = onSelect
        highWarningBps = GemstoneConfig.shared.swapConfig().highSlippageWarningBps
        let bps: UInt32
        switch slippage {
        case .auto:
            isAuto = true
            bps = Self.defaultBps
        case let .manual(value):
            isAuto = false
            bps = value
        }
        inputModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [PercentTextValidator(minimum: Self.minPercent, maximum: Self.maxPercent)],
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
        Self.parseBps(inputModel.text)
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
        Self.suggestionsBps.map { SlippageSuggestion(bps: $0, percentText: Self.format(bps: $0)) }
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
            maximumIntegerDigits: Self.maxIntegerDigits,
        ).sanitize(text)
    }

    func confirm() {
        onSelect(isAuto ? .auto : .manual(bps: selectedBps))
    }

    private static func format(bps: UInt32) -> String {
        (Double(bps) / 100).formatted(.number.precision(.fractionLength(0 ... 2)))
    }

    private static func parseBps(_ text: String) -> UInt32 {
        guard let percent = formatter.double(from: text), percent > 0 else { return 0 }
        return UInt32((min(percent, maxPercent) * 100).rounded())
    }
}
