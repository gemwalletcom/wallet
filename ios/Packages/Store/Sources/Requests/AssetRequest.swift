// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssetRequest: DatabaseQueryable {
    public var assetId: AssetId
    private let walletId: WalletId

    public init(
        walletId: WalletId,
        assetId: AssetId,
    ) {
        self.walletId = walletId
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> AssetData {
        guard let result = try AssetRecord.assetInfoRequest(walletId: walletId, assetId: assetId)
            .fetchOne(db)?
            .assetData
        else {
            throw AnyError("Asset not found: \(assetId.identifier)")
        }
        return result
    }
}

extension AssetRecord {
    static func assetInfoRequest(
        walletId: WalletId,
        assetId: AssetId?,
    ) -> QueryInterfaceRequest<AssetRecordInfo> {
        AssetRecord
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.balance.filter(BalanceRecord.Columns.walletId == walletId.id))
            .including(optional: AssetRecord.account.filter(AccountRecord.Columns.walletId == walletId.id))
            .including(all: AssetRecord.priceAlerts)
            .filter(AssetRecord.Columns.id == assetId?.identifier)
            .asRequest(of: AssetRecordInfo.self)
    }
}
