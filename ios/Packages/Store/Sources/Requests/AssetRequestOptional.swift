// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssetRequestOptional: DatabaseQueryable {
    public var assetId: AssetId?
    private let walletId: WalletId

    public init(
        walletId: WalletId,
        assetId: AssetId?,
    ) {
        self.walletId = walletId
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> AssetData? {
        try AssetRecord.assetInfoRequest(walletId: walletId, assetId: assetId)
            .fetchOne(db)?
            .assetData
    }
}

extension AssetRequestOptional: Equatable {}
