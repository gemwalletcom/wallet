// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

public struct PerpetualWalletBalanceRequest: DatabaseQueryable, Equatable {
    private let walletId: WalletId

    public init(walletId: WalletId) {
        self.walletId = walletId
    }

    public func fetch(_ db: Database) throws -> WalletBalance {
        let balance = try BalanceRecord
            .filter(BalanceRecord.Columns.walletId == walletId.id)
            .filter(BalanceRecord.Columns.assetId == Self.collateralAssetId)
            .fetchOne(db)
        guard let balance else { return .zero }
        return WalletBalance.perpetual(
            available: balance.availableAmount,
            reserved: balance.reservedAmount,
        )
    }

    private static let collateralAssetId = Asset.hypercoreUSDC().id.identifier
}
