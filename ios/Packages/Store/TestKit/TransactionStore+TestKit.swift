// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public extension TransactionStore {
    static func mock(db: DB = .mock()) -> TransactionStore {
        TransactionStore(db: db)
    }
}

public extension TransactionAssets {
    static func mock(_ transaction: Transaction, assetIds: [AssetId]? = .none) -> TransactionAssets {
        TransactionAssets(transaction: transaction, assetIds: assetIds ?? [transaction.assetId])
    }
}
