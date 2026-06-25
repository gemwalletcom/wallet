// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct AssetListStore: Sendable {
    private let dbQueue: DatabaseQueue

    public init(db: DB) {
        dbQueue = db.dbQueue
    }

    public func upsert(_ lists: [AssetList]) throws {
        try dbQueue.write { database in
            for list in lists {
                try list.record.upsert(database)
            }
        }
    }
}
