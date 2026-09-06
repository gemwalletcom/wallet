// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct NFTAssetRequest: DatabaseQueryable {
    private let walletId: WalletId
    private let assetId: NFTAssetId

    public init(walletId: WalletId, assetId: NFTAssetId) {
        self.walletId = walletId
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> NFTAssetDetails {
        guard let info = try NFTAssetRecord
            .filter(NFTAssetRecord.Columns.id == assetId.identifier)
            .including(required: NFTAssetRecord.collection.forKey("collection"))
            .including(all: NFTAssetRecord.assetAssociations.filter(NFTAssetAssociationRecord.Columns.walletId == walletId.id).forKey("associations"))
            .asRequest(of: NFTAssetRecordInfo.self)
            .fetchOne(db)
        else {
            throw AnyError("NFT asset not found: \(assetId.identifier)")
        }
        return info.mapToDetails()
    }
}

extension NFTAssetRequest: Equatable {}
