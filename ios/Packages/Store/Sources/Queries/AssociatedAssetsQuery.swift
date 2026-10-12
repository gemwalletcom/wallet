// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssociatedAssetsQuery: DatabaseQueryable {
    public let assetId: AssetId
    public let type: AssetAssociationType

    public init(assetId: AssetId, type: AssetAssociationType) {
        self.assetId = assetId
        self.type = type
    }

    public func fetch(_ db: Database) throws -> [Asset] {
        let associations = try AssetRecord
            .filter(AssetRecord.Columns.id == assetId.identifier)
            .fetchOne(db)?
            .associations ?? []
        let assetIds = associations.filter { $0.type == type }.map(\.assetId.identifier)
        return try AssetRecord
            .filter(assetIds.contains(AssetRecord.Columns.id))
            .fetchAll(db)
            .map { $0.mapToAsset() }
    }
}

extension AssociatedAssetsQuery: Equatable {}
