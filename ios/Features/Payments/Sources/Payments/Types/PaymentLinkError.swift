// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

enum PaymentLinkError: Error, Equatable {
    case noQuotes
    case quoteUnavailable
    case invalidDataCollectionUrl
    case dataCollection
    case unknownAsset
}

extension PaymentLinkError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .noQuotes, .quoteUnavailable, .invalidDataCollectionUrl: Localized.Errors.notSupported
        case .dataCollection, .unknownAsset: Localized.Errors.errorOccurred
        }
    }
}
