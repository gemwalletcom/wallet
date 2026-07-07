// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives

@Observable
public final class SwapSlippageViewModel {
    static let options: [UInt32] = [30, 50, 100, 300, 500]
    private static let defaultBps: UInt32 = 100

    private let onSelect: (SwapSlippage) -> Void
    private let highWarningBps: UInt32

    var isAuto: Bool
    var selectedBps: UInt32

    public init(slippage: SwapSlippage, onSelect: @escaping (SwapSlippage) -> Void) {
        self.onSelect = onSelect
        highWarningBps = GemstoneConfig.shared.swapConfig().highSlippageWarningBps
        switch slippage {
        case .auto:
            isAuto = true
            selectedBps = Self.defaultBps
        case let .manual(bps):
            isAuto = false
            selectedBps = bps
        }
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

    var selectedField: ListItemField {
        ListItemField(title: title, value: SwapSlippageSuggestion(bps: selectedBps).title)
    }

    var warningText: String? {
        guard selectedBps >= highWarningBps else { return nil }
        return Localized.Swap.slippageWarning
    }

    func apply() {
        onSelect(isAuto ? .auto : .manual(bps: selectedBps))
    }
}
