// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct PerpetualWalletBalance: Equatable, Sendable {
    public let balance: PerpetualBalance
    public let price: Double

    public init(balance: PerpetualBalance, price: Double) {
        self.balance = balance
        self.price = price
    }
}

public struct PerpetualWalletBalanceRequest: DatabaseQueryable, Equatable {
    private let walletId: WalletId
    private let assetId: AssetId

    public init(walletId: WalletId, assetId: AssetId) {
        self.walletId = walletId
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> PerpetualWalletBalance? {
        try BalanceRecord
            .filter(BalanceRecord.Columns.walletId == walletId.id)
            .filter(BalanceRecord.Columns.assetId == assetId.identifier)
            .fetchOne(db)
            .map {
                PerpetualWalletBalance(
                    balance: PerpetualBalance(available: $0.availableAmount, reserved: $0.reservedAmount, withdrawable: $0.withdrawableAmount),
                    price: (try? PriceRecord.filter(PriceRecord.Columns.assetId == assetId.identifier).fetchOne(db))??.price ?? 0,
                )
            }
    }
}
