import Foundation
import GRDB
import Primitives

public struct NFTStore: Sendable {
    private let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    // MARK: - Public methods

    public func getAsset(assetId: NFTAssetId) throws -> NFTAsset? {
        try db.read { db in
            try NFTAssetRecord
                .filter(NFTAssetRecord.Columns.id == assetId.identifier)
                .fetchOne(db)?
                .mapToAsset()
        }
    }

    public func getCollection(collectionId: NFTCollectionId) throws -> NFTCollection? {
        try db.read { db in
            try NFTCollectionRecord
                .filter(NFTCollectionRecord.Columns.id == collectionId.identifier)
                .fetchOne(db)?
                .mapToCollection()
        }
    }

    public func add(asset: NFTAsset, collection: NFTCollection) throws {
        try db.write { db in
            try collection.record().upsert(db)
            try asset.record().upsert(db)
        }
    }

    public func save(_ data: [NFTData], for walletId: WalletId) throws {
        try db.write { db in
            let assetsAssociationsRequest = NFTAssetAssociationRecord
                .filter(NFTAssetAssociationRecord.Columns.walletId == walletId.id)
            let existingIds = try assetsAssociationsRequest.fetchAll(db).map(\.id)

            var newIds: [String] = []

            for nftData in data {
                let collection = nftData.collection

                try collection.record().upsert(db)

                for asset in nftData.assets {
                    try asset.record().upsert(db)

                    let assetAssociation = NFTAssetAssociationRecord(walletId: walletId.id, collectionId: collection.id, assetId: asset.id)
                    try assetAssociation.upsert(db)

                    newIds.append(assetAssociation.id)
                }
            }

            let deleteIds = existingIds.asSet().subtracting(newIds.asSet()).asArray()
            try assetsAssociationsRequest
                .filter(deleteIds.contains(NFTAssetAssociationRecord.Columns.id))
                .deleteAll(db)
        }
    }
}
