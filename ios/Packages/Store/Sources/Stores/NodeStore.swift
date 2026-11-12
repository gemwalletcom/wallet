// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct NodeStore: Sendable {
    let db: DatabasePool

    public init(db: DB) {
        self.db = db.dbPool
    }

    public func addNode(chain: Chain, node: Node) throws {
        try db.write { db in
            try node
                .mapToRecord(chain: chain)
                .upsert(db)
        }
    }

    public func nodes(chain: Chain) throws -> [Node] {
        try db.read { db in
            try NodeRecord
                .filter(NodeRecord.Columns.chain == chain.rawValue)
                .fetchAll(db)
                .map { $0.mapToNode() }
        }
    }

    public func deleteNode(chain: Chain, url: String) throws {
        _ = try db.write { db in
            try NodeRecord
                .filter(NodeRecord.Columns.chain == chain.rawValue && NodeRecord.Columns.url == url)
                .deleteAll(db)
        }
    }
}
