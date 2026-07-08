// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PriceUsdRequest: DatabaseQueryable {
    public var assetId: AssetId

    public init(assetId: AssetId) {
        self.assetId = assetId
    }

    public func fetch(_ db: Database) throws -> Double? {
        try PriceRecord
            .filter(PriceRecord.Columns.assetId == assetId.identifier)
            .fetchOne(db)?
            .priceUsd
    }
}

extension PriceUsdRequest: Equatable {}
