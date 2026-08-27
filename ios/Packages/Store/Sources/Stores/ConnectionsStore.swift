// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct ConnectionsStore: Sendable {
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }

    // MARK: - Public methods

    public func addConnection(_ connection: WalletConnection) throws {
        try db.write { db in
            try connection.record.insert(db)
        }
    }

    public func getConnection(sessionId: String) throws -> WalletConnection? {
        try db.read { db in
            try WalletRecord
                .including(required: WalletRecord.connection)
                .asRequest(of: WalletConnectionInfo.self)
                .filter(
                    TableAlias(name: WalletConnectionRecord.databaseTableName)[WalletConnectionRecord.Columns.sessionId] == sessionId,
                )
                .fetchOne(db)?
                .mapToWalletConnection()
        }
    }

    public func getSessions() throws -> [WalletConnectionSession] {
        try db.read { db in
            try WalletConnectionRecord
                .fetchAll(db)
                .map(\.session)
        }
    }

    public func updateConnectionSession(_ session: WalletConnectionSession) throws {
        let connection = try getConnection(id: session.id).update(with: session)
        try db.write { db in
            try connection.upsert(db)
        }
    }

    public func delete(ids: [String]) throws -> Int {
        try db.write { db in
            try WalletConnectionRecord
                .filter(ids.contains(WalletConnectionRecord.Columns.id) || ids.contains(WalletConnectionRecord.Columns.sessionId))
                .deleteAll(db)
        }
    }

    public func deleteAll() throws -> Int {
        try db.write { db in
            try WalletConnectionRecord.deleteAll(db)
        }
    }

    // MARK: - Private methods

    private func getConnection(id: String) throws -> WalletConnectionRecord {
        try db.read { db in
            guard let connection = try WalletConnectionRecord
                .filter(WalletConnectionRecord.Columns.id == id || WalletConnectionRecord.Columns.sessionId == id)
                .fetchOne(db)
            else {
                throw AnyError("wallet connection record not found")
            }
            return connection
        }
    }
}
