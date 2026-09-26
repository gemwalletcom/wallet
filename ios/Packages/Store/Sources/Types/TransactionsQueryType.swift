// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum TransactionsQueryType: Equatable {
    case all
    case asset(assetId: AssetId)
    case transaction(id: String)
}

extension TransactionsQueryType: Identifiable {
    public var id: String {
        switch self {
        case .all: "all"
        case let .transaction(id): id
        case let .asset(asset): asset.identifier
        }
    }
}

extension TransactionsQueryType: Sendable {}
