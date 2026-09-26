// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionsQuery {
    static func assetScene(walletId: WalletId, assetId: AssetId, limit: Int) -> TransactionsQuery {
        TransactionsQuery(
            walletId: walletId,
            type: .asset(assetId: assetId),
            limit: limit,
        )
    }

    static func perpetualScene(walletId: WalletId, assetId: AssetId, types: [TransactionType], limit: Int) -> TransactionsQuery {
        TransactionsQuery(
            walletId: walletId,
            type: .asset(assetId: assetId),
            filter: TransactionsFilter(assetId: nil, chains: [], transactionTypes: types, states: [], assetRankGreaterThan: nil),
            limit: limit,
        )
    }
}
