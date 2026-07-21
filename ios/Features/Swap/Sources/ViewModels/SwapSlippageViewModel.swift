// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.SwapSlippageConfig
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

    private let config: SwapSlippageConfig
    private let onSelect: (SwapSlippage) -> Void

    var isAuto: Bool
    var inputModel: InputValidationViewModel
    var infoSheet: InfoSheetType?

    public init(
        slippage: SwapSlippage,
        defaultBps: UInt32,
        config: SwapSlippageConfig = SwapConfig.config().slippage,
        onSelect: @escaping (SwapSlippage) -> Void,
    ) {
        self.config = config
        self.onSelect = onSelect
        let bps: UInt32
        switch slippage {
        case .auto:
            isAuto = true
            bps = defaultBps
        case let .manual(value):
            isAuto = false
            bps = value
        }
        inputModel = InputValidationViewModel(
            mode: .onDemand,
            validators: [PercentTextValidator(minimum: Double(config.minBps) / 100, maximum: Double(config.maxBps) / 100)],
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
        guard inputModel.isValid, selectedBps >= config.highWarningBps else { return nil }
        return Localized.Swap.slippageWarning
    }

    var suggestions: [SlippageSuggestion] {
        config.suggestionsBps.map { SlippageSuggestion(bps: $0, percentText: Self.format(bps: $0)) }
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
            maximumIntegerDigits: maxIntegerDigits,
        ).sanitize(text)
    }

    func confirm() {
        onSelect(isAuto ? .auto : .manual(bps: selectedBps))
    }

    private var maxPercent: Double {
        Double(config.maxBps) / 100
    }

    private var maxIntegerDigits: Int {
        String(Int(maxPercent)).count
    }

    private static func format(bps: UInt32) -> String {
        (Double(bps) / 100).formatted(.number.precision(.fractionLength(0 ... 2)))
    }

    private func parseBps(_ text: String) -> UInt32 {
        guard let percent = Self.formatter.double(from: text), percent > 0 else { return 0 }
        return UInt32((min(percent, maxPercent) * 100).rounded())
    }
}
