// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct NodeRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName: String = "nodes"

    enum Columns {
        static let id = Column("id")
        static let url = Column("url")
        static let chain = Column("chain")
        static let status = Column("status")
        static let priority = Column("priority")
    }

    var id: Int?
    var url: String
    var chain: Chain
    var status: String
    var priority: Int
}

extension NodeRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: databaseTableName, ifNotExists: true) {
            $0.autoIncrementedPrimaryKey(Columns.id.name)
            $0.column(Columns.url.name, .text)
                .unique()
            $0.column(Columns.chain.name, .text)
                .notNull()
                .indexed()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.status.name, .text)
            $0.column(Columns.priority.name, .integer)
        }
    }
}

extension NodeRecord {
    func mapToChainNode() -> ChainNode {
        ChainNode(
            chain: chain.rawValue,
            node: mapToNode(),
        )
    }

    func mapToNode() -> Node {
        Node(
            url: url,
            status: NodeState(rawValue: status) ?? .inactive,
            priority: priority.asInt32,
        )
    }
}

extension Node {
    func mapToRecord(chain: Chain) -> NodeRecord {
        NodeRecord(
            url: url,
            chain: chain,
            status: status.rawValue,
            priority: Int(priority),
        )
    }
}
