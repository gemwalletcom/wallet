// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransactionsQueryFilter {
    case chains([String])
    case types([String])
    case assetRankGreaterThan(Int)
    case states([String])
}

extension TransactionsQueryFilter: Equatable {}
extension TransactionsQueryFilter: Sendable {}
