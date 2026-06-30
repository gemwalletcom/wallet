// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionsRequest {
    static func assetScene(walletId: WalletId, assetId: AssetId) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
        )
    }

    static func perpetualScene(walletId: WalletId, assetId: AssetId) -> TransactionsRequest {
        TransactionsRequest(
            walletId: walletId,
            type: .asset(assetId: assetId),
            filters: [.types([
                TransactionType.perpetualOpenPosition.rawValue,
                TransactionType.perpetualClosePosition.rawValue,
            ])],
        )
    }
}
