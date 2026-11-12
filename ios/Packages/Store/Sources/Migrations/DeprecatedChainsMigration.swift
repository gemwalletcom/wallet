// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

private struct ForeignKeyReference {
    let table: String
    let column: String
    let parentColumn: String
}

extension Database {
    func removeDeprecatedChains(_ chains: [String]) throws {
        for chain in chains {
            try removeDeprecatedChain(chain)
        }
    }

    private func removeDeprecatedChain(_ chain: String) throws {
        let wallets = WalletRecord.databaseTableName
        let accounts = AccountRecord.databaseTableName
        let assets = AssetRecord.databaseTableName
        try deleteCascading(
            from: wallets,
            where: "type = ? AND id IN (SELECT walletId FROM \(accounts) WHERE chain = ?)",
            arguments: [WalletType.view.rawValue, chain],
        )
        try deleteCascading(
            from: accounts,
            where: "chain = ? AND walletId IN (SELECT id FROM \(wallets) WHERE type = ?)",
            arguments: [chain, WalletType.multicoin.rawValue],
        )
        for reference in try foreignKeyReferences(to: assets) where reference.table != accounts {
            let column = reference.column.quotedDatabaseIdentifier
            try deleteCascading(from: reference.table, where: "\(column) = ? OR \(column) LIKE ? ESCAPE '\\'", arguments: [chain, "\(chain)\\_%"])
        }
        if try tableExists(ContactAddressRecord.databaseTableName) {
            try deleteCascading(from: ContactAddressRecord.databaseTableName, where: "chain = ?", arguments: [chain])
        }
        try deleteCascading(
            from: assets,
            where: "chain = ? AND (id != ? OR id NOT IN (SELECT chain FROM \(accounts)))",
            arguments: [chain, chain],
        )
    }

    private func deleteCascading(from table: String, where condition: String, arguments: StatementArguments) throws {
        let quotedTable = table.quotedDatabaseIdentifier
        let references = try foreignKeyReferences(to: table)
        guard !references.isEmpty else {
            try execute(sql: "DELETE FROM \(quotedTable) WHERE \(condition)", arguments: arguments)
            return
        }
        let rowIds = try Int64.fetchAll(self, sql: "SELECT rowid FROM \(quotedTable) WHERE \(condition)", arguments: arguments)
        guard !rowIds.isEmpty else { return }
        let selected = "rowid IN (\(rowIds.map(String.init).joined(separator: ", ")))"
        for reference in references {
            try deleteCascading(
                from: reference.table,
                where: "\(reference.column.quotedDatabaseIdentifier) IN (SELECT \(reference.parentColumn.quotedDatabaseIdentifier) FROM \(quotedTable) WHERE \(selected))",
                arguments: [],
            )
        }
        try execute(sql: "DELETE FROM \(quotedTable) WHERE \(selected)")
    }

    private func foreignKeyReferences(to parent: String) throws -> [ForeignKeyReference] {
        try String.fetchAll(self, sql: "SELECT name FROM sqlite_master WHERE type = 'table'").flatMap { table in
            try foreignKeys(on: table)
                .filter { $0.destinationTable == parent }
                .flatMap { $0.mapping.map { ForeignKeyReference(table: table, column: $0.origin, parentColumn: $0.destination) } }
        }
    }
}
