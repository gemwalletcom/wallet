// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemFiatQuotePhase
import enum Gemstone.GemFiatAmountCheck
import enum Gemstone.GemFiatButtonAction
import GemstonePrimitives
import Localization
import Primitives

extension GemFiatButtonAction {
    var title: String {
        switch self {
        case .continue: Localized.Common.continue
        case .retryQuote: Localized.Common.tryAgain
        }
    }
}

extension GemFiatAmountCheck {
    func errorText(locale: Locale) -> String? {
        switch self {
        case let .belowMinimum(minimum): Localized.Transfer.minimumAmount(minimum.text(locale: locale))
        case let .aboveMaximum(maximum): Localized.Transfer.maximumAmount(maximum.text(locale: locale))
        case let .insufficientBalance(title): Localized.Transfer.insufficientBalance(title.boldMarkdown())
        case .valid: nil
        }
    }
}

extension GemFiatQuotePhase {
    var inputErrorText: String? {
        switch self {
        case .invalidInput: Localized.Errors.invalidAmount
        case .noInput, .invalid, .loading, .ready, .noQuotes, .failed: nil
        }
    }
}
