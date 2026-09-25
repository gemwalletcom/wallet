// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSlippageFooter
import enum Gemstone.GemSwapButtonAction
import enum Gemstone.GemSwapErrorDisplay
import GemstonePrimitives
import Localization
import Primitives

extension GemSwapButtonAction {
    func title(symbol: String) -> String {
        switch self {
        case .retryQuote, .retryTransfer: Localized.Common.tryAgain
        case .insufficientBalance: Localized.Transfer.insufficientBalance(symbol)
        case .useMinimumAmount: Localized.Swap.useMinimumAmount
        case .swap: Localized.Wallet.swap
        }
    }
}

extension GemSwapErrorDisplay: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .notSupportedAsset: Localized.Errors.Swap.notSupportedAsset
        case .noQuote: Localized.Errors.Swap.noQuoteAvailable
        case .offline: Localized.Errors.networkOffline
        case let .minimumAmount(minimum):
            Localized.Errors.Swap.minimumAmount(minimum.text().boldMarkdown())
        case .amountTooSmall: Localized.Errors.Swap.amountTooSmall
        }
    }
}

extension GemSlippageFooter {
    var text: String {
        switch self {
        case let .minimum(value): Localized.Common.minimumValue(value.text())
        case let .maximum(value): Localized.Common.maximumValue(value.text())
        case .warning: Localized.Swap.slippageWarning
        }
    }
}
