// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct NFTAssetQuery: DatabaseQueryable {
    private let walletId: WalletId
    private let assetId: NFTAssetId

    public init(walletId: WalletId, assetId: NFTAssetId) {
        self.walletId = walletId
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> NFTAssetDetails? {
        try NFTAssetRecord
            .filter(NFTAssetRecord.Columns.id == assetId.identifier)
            .including(required: NFTAssetRecord.collection.forKey("collection"))
            .including(all: NFTAssetRecord.assetAssociations.filter(NFTAssetAssociationRecord.Columns.walletId == walletId.id).forKey("associations"))
            .asRequest(of: NFTAssetRecordInfo.self)
            .fetchOne(db)?
            .mapToDetails()
    }
}

extension NFTAssetQuery: Equatable {}
