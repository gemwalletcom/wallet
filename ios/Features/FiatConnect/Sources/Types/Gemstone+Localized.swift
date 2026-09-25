// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemFiatAmountError
import enum Gemstone.GemFiatButtonAction
import enum Gemstone.GemFiatQuotesMessage
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

extension GemFiatButtonAction {
    var title: String {
        switch self {
        case .continue: Localized.Common.continue
        case .retryQuote: Localized.Common.tryAgain
        }
    }
}

extension GemFiatAmountError {
    func text(locale: Locale) -> String {
        switch self {
        case .invalidAmount: Localized.Errors.invalidAmount
        case let .belowMinimum(minimum): Localized.Transfer.minimumAmount(minimum.text(locale: locale))
        case let .aboveMaximum(maximum): Localized.Transfer.maximumAmount(maximum.text(locale: locale))
        case let .insufficientBalance(title): Localized.Transfer.insufficientBalance(title.boldMarkdown())
        }
    }
}

extension GemFiatQuotesMessage {
    func title(action: String) -> String {
        switch self {
        case .enterAmount: Localized.Input.enterAmountTo(action)
        case .noResults: Localized.Buy.noResults
        case let .failed(error): error.text
        }
    }
}

