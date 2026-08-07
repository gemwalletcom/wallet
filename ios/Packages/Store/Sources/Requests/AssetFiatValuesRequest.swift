// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct AssetFiatValuesRequest: DatabaseQueryable, Equatable {
    public var walletId: WalletId
    public var type: TotalValueType
    public var perpetualAccountMode: PerpetualAccountMode

    public init(walletId: WalletId, type: TotalValueType, perpetualAccountMode: PerpetualAccountMode = .standard) {
        self.walletId = walletId
        self.type = type
        self.perpetualAccountMode = perpetualAccountMode
    }

    public func fetch(_ db: Database) throws -> [AssetFiatValue] {
        switch type {
        case .perpetual:
            return try [perpetualFiatValue(db)]
        case .wallet:
            let assets = try assetRecords(db).compactMap {
                AssetFiatValue(record: $0, amount: $0.balance.totalAmount)
            }
            switch perpetualAccountMode {
            case .unified: return assets
            case .standard: return try assets + [perpetualFiatValue(db)]
            }
        case .earn:
            return try assetRecords(db).compactMap {
                AssetFiatValue(record: $0, amount: $0.balance.stakedAmount + $0.balance.earnAmount)
            }
        }
    }

    private func assetRecords(_ db: Database) throws -> [AssetRecordInfoMinimal] {
        try AssetRecord
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.balance)
            .joining(required: AssetRecord.balance
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.isEnabled == true))
            .asRequest(of: AssetRecordInfoMinimal.self)
            .fetchAll(db)
    }

    private func perpetualFiatValue(_ db: Database) throws -> AssetFiatValue {
        let balance = try PerpetualWalletBalanceRequest(walletId: walletId).fetch(db)
        return AssetFiatValue(amount: balance.total, price: 1, priceChangePercentage24h: 0)
    }
}

extension AssetFiatValue {
    init?(record: AssetRecordInfoMinimal, amount: Double) {
        guard let price = record.price, price.price > 0 else { return nil }
        self.init(
            amount: amount,
            price: price.price,
            priceChangePercentage24h: price.priceChangePercentage24h,
        )
    }
}
