// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PriceQuery: DatabaseQueryable {
    public var assetId: AssetId

    public init(assetId: AssetId) {
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> PriceData? {
        try AssetRecord
            .including(optional: AssetRecord.price)
            .including(optional: AssetRecord.market)
            .including(all: AssetRecord.priceAlerts)
            .including(all: AssetRecord.links)
            .filter(AssetRecord.Columns.id == assetId.identifier)
            .asRequest(of: PriceRecordInfo.self)
            .fetchOne(db)
            .map(\.priceData)
    }
}

extension PriceQuery: Equatable {}
