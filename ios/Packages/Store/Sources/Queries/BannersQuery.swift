// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct BannersQuery: DatabaseQueryable {
    public init() {}

    public func fetch(_ db: Database) throws -> [Banner] {
        try BannerRecord
            .including(optional: BannerRecord.asset)
            .asRequest(of: BannerInfo.self)
            .fetchAll(db)
            .map { $0.mapToBanner() }
    }
}

extension BannersQuery: Equatable {}
