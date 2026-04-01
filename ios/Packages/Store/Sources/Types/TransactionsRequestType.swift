// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum TransactionsRequestType: Equatable {
    case all
    case pending
    case asset(assetId: AssetId)
    case assetsTransactionType(assetIds: [AssetId], type: TransactionType, states: [TransactionState])
    case transaction(id: String)
}

extension TransactionsRequestType: Identifiable {
    public var id: String {
        switch self {
        case .all: "all"
        case .pending: "pending"
        case let .transaction(id): id
        case let .asset(asset): asset.identifier
        case let .assetsTransactionType(assetIds, type, _):
            assetIds.map(\.identifier).joined() + type.rawValue
        }
    }
}

extension TransactionsRequestType: Sendable {}
