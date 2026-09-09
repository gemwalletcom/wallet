// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct AssetFiatValuesRequest: DatabaseQueryable, Equatable {
    public var walletId: WalletId
    public var type: TotalValueType
    public var perpetualAssetId: AssetId

    public init(walletId: WalletId, type: TotalValueType, perpetualAssetId: AssetId) {
        self.walletId = walletId
        self.type = type
        self.perpetualAssetId = perpetualAssetId
    }

    public func fetch(_ db: Database) throws -> [AssetFiatValue] {
        switch type {
        case .perpetual:
            return try [perpetualFiatValue(db)]
        case .wallet:
            return try assetRecords(db).map {
                AssetFiatValue(record: $0, amount: $0.balance.totalAmount)
            }
        case .earn:
            return try assetRecords(db).map {
                AssetFiatValue(record: $0, amount: $0.balance.stakedAmount + $0.balance.earnAmount)
            }
        }
    }

    private func assetRecords(_ db: Database) throws -> [AssetRecordInfoMinimal] {
        try AssetRecord
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.balance)
            .filter(AssetRecord.Columns.rank >= 0)
            .joining(required: AssetRecord.balance
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.isEnabled == true))
            .asRequest(of: AssetRecordInfoMinimal.self)
            .fetchAll(db)
    }

    private func perpetualFiatValue(_ db: Database) throws -> AssetFiatValue {
        let balance = try PerpetualWalletBalanceRequest(walletId: walletId, assetId: perpetualAssetId).fetch(db)
        return AssetFiatValue(amount: balance.map { $0.available + $0.reserved } ?? 0, price: 1, priceChangePercentage24h: 0)
    }
}

extension AssetFiatValue {
    init(record: AssetRecordInfoMinimal, amount: Double) {
        self.init(
            amount: amount,
            price: record.price?.price ?? 0,
            priceChangePercentage24h: record.price?.priceChangePercentage24h ?? 0,
        )
    }
}
