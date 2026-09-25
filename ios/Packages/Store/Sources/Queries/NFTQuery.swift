// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public enum NFTFilter: Sendable, Hashable {
    case all
    case collection(id: String)
}

public struct NFTQuery: DatabaseQueryable {
    private let walletId: WalletId
    private let filter: NFTFilter

    public init(walletId: WalletId, filter: NFTFilter) {
        self.walletId = walletId
        self.filter = filter
    }

    public func fetch(_ db: Database) throws -> [NFTData] {
        let walletAssets = NFTCollectionRecord.assets
            .joining(
                required: NFTAssetRecord.assetAssociations
                    .filter(NFTAssetAssociationRecord.Columns.walletId == walletId.id),
            )
        var request = NFTCollectionRecord
            .joining(required: walletAssets.forKey("walletAssets"))
            .including(all: walletAssets)
            .distinct()
            .asRequest(of: NFTCollectionRecordInfo.self)

        switch filter {
        case .all: break
        case let .collection(id): request = request.filter(NFTCollectionRecord.Columns.id == id)
        }

        return try request
            .fetchAll(db)
            .map { $0.mapToNFTData() }
    }
}

extension NFTQuery: Equatable {}
