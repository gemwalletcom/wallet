// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

public enum PaymentLinkError: Error, Equatable {
    case noQuotes
    case quoteUnavailable
    case invalidDataCollectionUrl
    case unknownAsset
}

extension PaymentLinkError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .noQuotes, .quoteUnavailable, .invalidDataCollectionUrl: Localized.Errors.notSupported
        case .unknownAsset: Localized.Errors.errorOccurred
        }
    }
}
