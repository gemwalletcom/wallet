// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Primitives

struct PositionRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "positions"

    enum Columns {
        static let id = Column("id")
    }

    var id: String
    var walletId: String
    var leverage: Int
    var entryPrice: Double
    var takeProfit: TriggerOrder?
    var previewImageUrl: String
    var tags: [String]?
    var state: ConnectionState?
    var updatedAt: Date
    var isPinned: Bool = false

    var identifier: String {
        id
    }
}
