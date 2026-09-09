// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct AssetFiatValuesRequest: DatabaseQueryable, Equatable {
    public var walletId: WalletId

    public init(walletId: WalletId) {
        self.walletId = walletId
    }

    public func fetch(_ db: Database) throws -> [AssetFiatValue] {
        try AssetRecord
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.balance)
            .filter(AssetRecord.Columns.rank >= 0)
            .joining(required: AssetRecord.balance
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.isEnabled == true))
            .asRequest(of: AssetRecordInfoMinimal.self)
            .fetchAll(db)
            .map {
                AssetFiatValue(
                    amount: $0.balance.totalAmount,
                    price: $0.price?.price ?? 0,
                    priceChangePercentage24h: $0.price?.priceChangePercentage24h ?? 0,
                )
            }
    }
}
