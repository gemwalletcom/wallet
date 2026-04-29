// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

enum ScanTransactionError: Error, Equatable {
    case malicious
    case memoRequired(symbol: String)
}

extension ScanTransactionError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .malicious: Localized.Errors.ScanTransaction.Malicious.description
        case let .memoRequired(symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        }
    }
}
