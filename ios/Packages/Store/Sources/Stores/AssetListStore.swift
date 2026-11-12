// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssetListStore: Sendable {
    private let db: DatabasePool

    public init(db: DB) {
        self.db = db.dbPool
    }

    public func upsert(_ lists: [AssetList]) throws {
        try db.write { database in
            for list in lists {
                try list.record.upsert(database)
            }
        }
    }
}
