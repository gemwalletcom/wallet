// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionsRequest {
    static func assetScene(walletId: WalletId, assetId: AssetId, limit: Int) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
            limit: limit,
        )
    }

    static func perpetualScene(walletId: WalletId, assetId: AssetId, types: [TransactionType], limit: Int) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
            filters: [.types(types.map(\.rawValue))],
            limit: limit,
        )
    }
}
